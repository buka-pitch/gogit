use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use crossterm::style::Stylize;

pub struct HookManager;

impl HookManager {
    fn get_hook_path() -> std::io::Result<PathBuf> {
        // Assume user is in repo root or we should use git rev-parse --git-dir
        // For simplified MVP, assume .git/hooks/prepare-commit-msg
        Ok(Path::new(".git").join("hooks").join("prepare-commit-msg"))
    }

    pub fn install() -> std::io::Result<()> {
        let path = Self::get_hook_path()?;
        
        if path.exists() {
             println!("{} Hook already exists at {:?}", "⚠".yellow(), path);
             println!("Please remove it manually if you want to overwrite it.");
             return Ok(());
        }
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy();

        let script = format!(r#"#!/bin/sh
# gogit: AI-powered commit generation
# Intercepts manual 'git commit' calls
exec "{}" commit "$@" < /dev/tty
"#, exe_path_str);
        
        fs::write(&path, script)?;
        
        // Make executable
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
        
        println!("{} Hook installed to {:?}", "✔".green(), path);
        println!("Try running `git commit`!");
        
        Ok(())
    }

    pub fn uninstall() -> std::io::Result<()> {
        let path = Self::get_hook_path()?;
        
        if path.exists() {
            fs::remove_file(&path)?;
             println!("{} Hook removed from {:?}", "✔".green(), path);
        } else {
             println!("{} No hook found at {:?}", "ℹ".blue(), path);
        }
        Ok(())
    }
}
