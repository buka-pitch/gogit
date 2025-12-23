use crate::ai::AiClient;
use crate::git::GitRepo;
use crate::tui::Tui;
use crossterm::style::Stylize;
use std::process::Command;

pub struct HistorySearcher;

impl HistorySearcher {
    pub async fn run(
        tui: &Tui,
        ai: &AiClient,
        _repo: &GitRepo,
        query: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spinner = tui.start_thinking("Scanning recent history...");
        
        // Fetch last 30 commits with their hashes and messages
        let output = Command::new("git")
            .arg("log")
            .arg("-n")
            .arg("30")
            .arg("--pretty=format:%h|%s")
            .output()?;

        if !output.status.success() {
            tui.stop_spinner(spinner);
            return Err("Failed to fetch git log".into());
        }

        let log_text = String::from_utf8(output.stdout)?;
        let lines: Vec<&str> = log_text.lines().collect();

        if lines.is_empty() {
            tui.stop_spinner(spinner);
            println!("No history found.");
            return Ok(());
        }

        let system_prompt = "You are a git search assistant. \
            Given a list of commit hashes and messages, find the most relevant commit to the user's query. \
            Output ONLY the commit hash of the best match. If nothing is relevant, output 'NONE'.";

        let prompt = format!(
            "QUERY: {}\n\nHISTORY:\n{}",
            query, log_text
        );

        let best_match = ai.generate_text(&prompt, system_prompt).await?;
        tui.stop_spinner(spinner);

        let hash = best_match.trim();
        if hash == "NONE" {
            println!("\n{} No relevant commits found for '{}'.", "ℹ".blue(), query);
        } else {
            println!("\n{} Found relevant commit: {}", "✔".green().bold(), hash.cyan());
            println!("{}", "--- COMMIT DETAILS ---".yellow().bold());
            
            // Show the commit
            let status = Command::new("git")
                .arg("show")
                .arg("--stat")
                .arg(hash)
                .status()?;

            if !status.success() {
                println!("{}", "Could not display commit details.".red());
            }
        }

        Ok(())
    }
}
