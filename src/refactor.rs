use crate::ai::AiClient;
use crate::tui::Tui;
use std::fs;
use std::path::Path;
use crossterm::style::Stylize;

pub struct Refactorer;

impl Refactorer {
    pub async fn run(
        tui: &Tui,
        ai: &AiClient,
        file_path: &str,
        instruction: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        
        // Optimization: Check for massive files
        if content.len() > 100_000 {
            println!("\n{} This file is very large ({} chars). API may be slow or expensive.", "⚠️".yellow(), content.len());
            if !tui.prompt_yes_no("Proceed anyway?")? {
                return Ok(());
            }
        }

        let spinner = tui.start_thinking(&format!("Refactoring {}...", file_path));

        let system_prompt = "You are an expert refactoring tool. \
            Your goal is to apply the requested changes to the provided code. \
            Output ONLY the raw content of the entire file after the changes. \
            Do not include markdown code blocks, explanations, or preamble.";

        let prompt = format!(
            "REQUESTED CHANGE: {}\n\nFILE CONTENT:\n\n{}",
            instruction, content
        );

        let new_content = ai.generate_text(&prompt, system_prompt).await?;
        tui.stop_spinner(spinner);

        // Strip any potential backticks just in case
        let clean_content = if new_content.trim().starts_with("```") {
             new_content.lines()
                .skip(1)
                .filter(|l| !l.trim().starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string()
        } else {
            new_content.clone()
        };

        if clean_content.trim().is_empty() {
             return Err("AI returned empty content. Refactoring aborted.".into());
        }

        println!("\n{}", "--- PROPOSED CHANGES ---".yellow().bold());
        // For simplicity, we just show a few lines or prompt to apply
        // A full diff view would be better, but let's start with a confirmation.
        
        if tui.prompt_yes_no(&format!("Apply changes to {}?", file_path))? {
            fs::write(path, clean_content)?;
            println!("\n{} {} updated.", "✔".green().bold(), file_path);
        } else {
            println!("\n{}", "Aborted.".red());
        }

        Ok(())
    }
}
