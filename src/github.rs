use octocrab::Octocrab;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("Octocrab error: {0}")]
    Octocrab(#[from] octocrab::Error),
    #[error("Environment variable missing: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Git error: {0}")]
    Git(String),
}

pub struct GitHub {
    client: Octocrab,
}

impl GitHub {
    pub fn new() -> Result<Self, GitHubError> {
        let token = env::var("GITHUB_TOKEN").or_else(|_| env::var("GH_TOKEN"))?;
        
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;
            
        Ok(Self { client })
    }

    pub async fn create_pr(&self, title: &str, body: &str, head: &str, base: &str, repo_owner: &str, repo_name: &str) -> Result<String, GitHubError> {
        let pr = self.client.pulls(repo_owner, repo_name)
            .create(title, head, base)
            .body(body)
            .send()
            .await?;
            
        Ok(pr.html_url.map(|u| u.to_string()).unwrap_or_default())
    }
}
