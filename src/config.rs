use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: String,
    pub commit_prompt: Option<String>,
    pub pr_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gemini-2.5-flash-lite".to_string(),
            commit_prompt: None,
            pr_prompt: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str(&contents) {
                        return config;
                    }
                }
            }
        }
        
        Self::default()
    }

    fn get_config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("gogit");
        path.push("config.toml");
        Some(path)
    }
}
