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
        // git log main..HEAD --no-merges --pretty=format:"%h %s%n%b"
        let output = Command::new("git")
            .arg("log")
            .arg(format!("{}..HEAD", main_branch))
            .arg("--no-merges")
            .arg("--pretty=format:Commit: %h%nMessage: %s%nBody: %b%n---")
            .output()?;

        if !output.status.success() {
             return Err(GitError::Cmd(String::from_utf8_lossy(&output.stderr).to_string()));
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
}
