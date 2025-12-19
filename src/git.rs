use std::path::Path;
use std::process::Command;
use thiserror::Error;
// removed unused Entry

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git discovery error: {0}")]
    Discovery(#[from] gix::discover::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Utf8 slice error: {0}")]
    Utf8Str(#[from] std::str::Utf8Error),
    #[error("Git command failed: {0}")]
    Cmd(String),
}

pub struct GitRepo {
    repo: gix::Repository,
}

pub struct MergeResult {
    pub has_conflicts: bool,
    pub conflicted_files: Vec<String>,
}

impl GitRepo {
    pub fn open() -> Result<Self, GitError> {
        let repo = gix::discover(".")?;
        Ok(Self { repo })
    }

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

    // Returns (owner, repo_name)
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

    pub fn get_repo_tree(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .arg("ls-files")
            .output()?;

        if !output.status.success() {
            return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

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
