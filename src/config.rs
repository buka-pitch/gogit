use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    pub commit_prompt: Option<String>,
    pub pr_prompt: Option<String>,
    pub max_stale_branches: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gemini-3-flash-preview".to_string(),
            provider: "gemini".to_string(),
            api_key: None,
            gemini_api_key: None,
            ollama_base_url: Some("http://localhost:11434".to_string()),
            commit_prompt: None,
            pr_prompt: None,
            max_stale_branches: Some(20),
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path().ok_or("Could not determine config directory")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        fs::write(&config_path, toml_string)?;

        Ok(())
    }
}
