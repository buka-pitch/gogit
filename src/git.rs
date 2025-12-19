use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Represents errors that can occur during Git operations.
#[derive(Error, Debug)]
pub enum GitError {
    /// An error occurred during Git repository discovery.
    #[error("Git discovery error: {0}")]
    Discovery(#[from] gix::discover::Error),
    /// An input/output error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A UTF-8 conversion error occurred.
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    /// A UTF-8 slice conversion error occurred.
    #[error("Utf8 slice error: {0}")]
    Utf8Str(#[from] std::str::Utf8Error),
    /// A Git command failed to execute successfully.
    #[error("Git command failed: {0}")]
    Cmd(String),
}

/// Represents a Git repository and provides methods for interacting with it.
pub struct GitRepo {
    repo: gix::Repository,
}

/// Represents the result of a merge operation, indicating whether conflicts occurred
/// and which files are in conflict.
pub struct MergeResult {
    /// `true` if merge conflicts were detected, `false` otherwise.
    pub has_conflicts: bool,
    /// A list of file paths that are in conflict.
    pub conflicted_files: Vec<String>,
}

impl GitRepo {
    /// Opens the Git repository in the current directory.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GitRepo` instance if successful, or a `GitError` if the repository cannot be opened.
    pub fn open() -> Result<Self, GitError> {
        let repo = gix::discover(".")?;
        Ok(Self { repo })
    }

    /// Retrieves the diff of staged changes in the Git repository.
    ///
    /// This method calculates the difference between the staged index and the last commit.
    /// It excludes files that are typically ignored by package managers or are binary assets.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` representing the diff in the standard Git diff format if successful,
    /// or a `GitError` if any Git command or file operation fails. Returns an empty string if there are no staged changes.
    pub fn get_staged_diff(&self) -> Result<String, GitError> {
        let repo = &self.repo;
        let index = repo.open_index().map_err(|e| GitError::Cmd(e.to_string()))?; // gix 0.66 uses open_index

        let mut staged_files = Vec::new();
        
        // Iterate over index entries to find what matches our filter
        for entry in index.entries() {
            let path = std::str::from_utf8(entry.path(&index))?; // gix < 0.6x might be different, but 0.66 path() takes &State
            if !self.is_ignored(path) {
                staged_files.push(path.to_string());
            }
        }

        if staged_files.is_empty() {
            return Ok(String::new());
        }

        // We use git diff --cached -- <files> to get the actual diff content
        // This is safer and robust for generating strict diffs for the LLM
        let mut cmd = Command::new("git");
        cmd.arg("diff").arg("--cached").arg("--");
        cmd.args(&staged_files);

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Checks if a given file path should be ignored.
    ///
    /// This method currently ignores common lock files (`package-lock.json`, `yarn.lock`, `Cargo.lock`, `pnpm-lock.yaml`, `go.sum`)
    /// and common binary file extensions (`.map`, `.svg`, `.png`, `.jpg`).
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the file path to check.
    ///
    /// # Returns
    ///
    /// `true` if the file should be ignored, `false` otherwise.
    fn is_ignored(&self, path: &str) -> bool {
        let p = Path::new(path);
        
        // Filter lockfiles
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name == "package-lock.json" 
                || name == "yarn.lock" 
                || name == "Cargo.lock" 
                || name == "pnpm-lock.yaml" 
                || name == "go.sum" {
                return true;
            }
            if name.ends_with(".map") || name.ends_with(".svg") || name.ends_with(".png") || name.ends_with(".jpg") {
                 return true;
            }
        }
        
        // Can add more logic here (e.g. check for minified files)
        false
    }

    /// Commits staged changes with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the commit was successful, or a `GitError` if the commit failed.
    pub fn commit(&self, message: &str) -> Result<(), GitError> {
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()?;
        
        if !output.status.success() {
             return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
         }
         Ok(())
    }

    /// Pushes staged changes to the remote repository.
    ///
    /// If the current branch does not have an upstream tracking branch, it attempts to set one up.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the push was successful, or a `GitError` if the push failed.
    pub fn push(&self) -> Result<(), GitError> {
         let output = Command::new("git")
            .arg("push")
            .output()?;
        
        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             if stderr.contains("no upstream branch") {
                 let current_branch = self.get_current_branch()?;
                 println!("Tip: Setting upstream for branch '{}'...", current_branch);
                 let status_retry = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg(&current_branch)
                    .status()?;
                
                if !status_retry.success() {
                     return Err(GitError::Cmd("Failed to push cleanup upstream".to_string()));
                }
             } else {
                 return Err(GitError::Cmd(stderr.to_string()));
             }
         }
         Ok(())
    }

    /// Retrieves commit history relevant for a Pull Request context.
    ///
    /// Fetches the latest changes for the specified `main_branch` from `origin` and then logs commits
    /// between the `main_branch` on origin and the current `HEAD`. It aims to provide a concise
    /// summary of recent commits that are not yet part of the main branch.
    ///
    /// # Arguments
    ///
    /// * `main_branch` - The name of the main branch (e.g., "main" or "master").
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` with the commit history formatted as "Commit: %h\nMessage: %s\nBody: %b\n---"
    /// if successful, or a `GitError` if any Git command fails.
    pub fn get_pr_context(&self, main_branch: &str) -> Result<String, GitError> {
        // Fetch the latest from origin for the base branch to ensure context is up to date
        // Note: This is non-destructive to local branches.
        println!("Fetching latest from origin/{} for context...", main_branch);
        let _ = Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg(main_branch)
            .status();

        // Use origin/main_branch instead of local branch to avoid stale context
        let base_ref = format!("origin/{}", main_branch);
        
        let output = Command::new("git")
            .arg("log")
            .arg(format!("{}..HEAD", base_ref))
            .arg("--no-merges")
            .arg("--pretty=format:Commit: %h%nMessage: %s%nBody: %b%n---")
            .output()?;

        if !output.status.success() {
             // Fallback to local if origin doesn't exist/fails
             let output_local = Command::new("git")
                .arg("log")
                .arg(format!("{}..HEAD", main_branch))
                .arg("--no-merges")
                .arg("--pretty=format:Commit: %h%nMessage: %s%nBody: %b%n---")
                .output()?;

             if !output_local.status.success() {
                 return Err(GitError::Cmd(String::from_utf8_lossy(&output_local.stderr).to_string()));
             }
             return Ok(String::from_utf8(output_local.stdout)?);
         }
         Ok(String::from_utf8(output.stdout)?)
    }

    /// Retrieves a list of files that have unstaged changes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<String>` of unstaged file paths if successful,
    /// or a `GitError` if the Git command fails.
    pub fn get_unstaged_files(&self) -> Result<Vec<String>, GitError> {
        // git diff --name-only
        let output = Command::new("git")
            .arg("diff")
            .arg("--name-only")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        let output_str = String::from_utf8(output.stdout)?;
        let files: Vec<String> = output_str.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|s| s.to_string())
            .collect();
            
        Ok(files)
    }

    /// Stages the specified files for the next commit.
    ///
    /// # Arguments
    ///
    /// * `files` - A slice of strings, where each string is the path to a file to be staged.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the files were staged successfully, or a `GitError` if the operation failed.
    /// If the `files` slice is empty, this function returns `Ok(())` immediately without executing any commands.
    pub fn stage_files(&self, files: &[String]) -> Result<(), GitError> {
        if files.is_empty() { return Ok(()); }
        
        let mut cmd = Command::new("git");
        cmd.arg("add");
        cmd.args(files);
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        Ok(())
    }

    /// Gets the name of the current Git branch.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` with the current branch name if successful,
    /// or a `GitError` if the command fails.
    pub fn get_current_branch(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()?;
            
        if !output.status.success() {
             return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
         }
         Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Retrieves the owner and repository name from the 'origin' remote URL.
    ///
    /// Supports both HTTPS and SSH URL formats.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple `(String, String)` representing the owner and repository name,
    /// or a `GitError` if the remote URL cannot be fetched or parsed.
    pub fn get_remote_info(&self) -> Result<(String, String), GitError> {
        let output = Command::new("git")
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .output()?;
            
        if !output.status.success() {
             return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
         }
         
         let url = String::from_utf8(output.stdout)?.trim().to_string();
         // Parse URL: 
         // https://github.com/owner/repo.git or git@github.com:owner/repo.git
         
         let parts: Vec<&str> = if url.starts_with("git@") {
             url.trim_start_matches("git@").split(':').nth(1).unwrap_or("").split('/').collect()
         } else {
             url.trim_start_matches("https://").split('/').skip(1).collect()
         };
         
         if parts.len() >= 2 {
             let owner = parts[0];
             let repo = parts[1].trim_end_matches(".git");
             Ok((owner.to_string(), repo.to_string()))
         } else {
             Err(GitError::Cmd("Could not parse remote url".to_string()))
         }
    }

    /// Checks for merge conflicts between two references.
    ///
    /// This function uses `git merge-tree --write-tree` to determine if a merge would result in conflicts.
    ///
    /// # Arguments
    ///
    /// * `base` - The base reference (e.g., a branch name or commit hash).
    /// * `head` - The head reference (e.g., a branch name or commit hash) to merge into the base.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `MergeResult` struct indicating the presence of conflicts and a list of conflicted files.
    /// Returns a `GitError` if the `git merge-tree` command fails.
    pub fn check_merge_conflicts(&self, base: &str, head: &str) -> Result<MergeResult, GitError> {
        let mut cmd = Command::new("git");
        cmd.arg("merge-tree")
           .arg("--write-tree")
           .arg(base)
           .arg(head);

        let output = cmd.output()?;
        
        let has_conflicts = match output.status.code() {
            Some(0) => false,
            Some(1) => true,
            _ => return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string())),
        };

        let mut conflicted_files = Vec::new();
        if has_conflicts {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if parts[2] == "1" || parts[2] == "2" || parts[2] == "3" {
                        let path = parts[3].to_string();
                        if !conflicted_files.contains(&path) {
                            conflicted_files.push(path);
                        }
                    }
                }
            }
        }

        Ok(MergeResult {
            has_conflicts,
            conflicted_files,
        })
    }

    /// Gets the most recent Git tag reachable from the current commit.
    ///
    /// If no tags are found, it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<String>` with the latest tag name if found,
    /// or `None` if no tags exist. Returns a `GitError` if the command fails for reasons other than no tags.
    pub fn get_last_tag(&self) -> Result<Option<String>, GitError> {
        let output = Command::new("git")
            .arg("describe")
            .arg("--tags")
            .arg("--abbrev=0")
            .output()?;

        if output.status.success() {
            Ok(Some(String::from_utf8(output.stdout)?.trim().to_string()))
        } else {
            // If no tags found, this command usually fails with exit code 128
            Ok(None)
        }
    }

    /// Gets a list of commits since a specified reference point.
    ///
    /// If no reference is provided, it lists all commits from the beginning of the repository history up to HEAD.
    /// It excludes merge commits and formats the output as a list with commit subjects and short SHAs.
    ///
    /// # Arguments
    ///
    /// * `reference` - An `Option<&str>` representing the reference commit or branch to compare against.
    ///                 If `None`, all commits are considered.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` with the formatted commit history if successful,
    /// or a `GitError` if the Git command fails.
    pub fn get_commits_since_ref(&self, reference: Option<&str>) -> Result<String, GitError> {
        let mut cmd = Command::new("git");
        cmd.arg("log");
        
        if let Some(r) = reference {
            cmd.arg(format!("{}..HEAD", r));
        }

        cmd.arg("--pretty=format:- %s (%h)")
           .arg("--no-merges");

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Lists all files tracked by Git in the repository.
    ///
    /// This is equivalent to running `git ls-files`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` with a list of tracked file paths, one per line, if successful.
    /// Returns a `GitError` if the Git command fails.
    pub fn get_repo_tree(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .arg("ls-files")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Retrieves a list of all local and remote branches.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<String>` of branch names if successful,
    /// or a `GitError` if the Git command fails.
    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let output = Command::new("git")
            .arg("branch")
            .arg("-a")
            .arg("--format=%(refname:short)")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        let output_str = String::from_utf8(output.stdout)?;
        Ok(output_str.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
    }

    /// Gets a list of branches that have been merged into a specified base branch.
    ///
    /// # Arguments
    ///
    /// * `base` - The name of the base branch to check against.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<String>` of merged branch names (excluding the base branch itself) if successful,
    /// or a `GitError` if the Git command fails.
    pub fn get_merged_branches(&self, base: &str) -> Result<Vec<String>, GitError> {
        let output = Command::new("git")
            .arg("branch")
            .arg("--merged")
            .arg(base)
            .arg("--format=%(refname:short)")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        let output_str = String::from_utf8(output.stdout)?;
        Ok(output_str.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != base)
            .collect())
    }

    /// Retrieves a summary of the latest commits on a given branch compared to a base branch.
    ///
    /// This function fetches the last 5 commits from `branch` that are not in `base`.
    ///
    /// # Arguments
    ///
    /// * `branch` - The name of the branch to get the summary from.
    /// * `base` - The name of the base branch to compare against.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` with the summarized commit information if successful,
    /// or a `GitError` if the Git command fails.
    pub fn get_branch_summary(&self, branch: &str, base: &str) -> Result<String, GitError> {
        let output = Command::new("git")
            .arg("log")
            .arg(format!("{}..{}", base, branch))
            .arg("--oneline")
            .arg("-n")
            .arg("5")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}