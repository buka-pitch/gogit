use crate::ai::GeminiClient;
use crate::git::GitRepo;
use crate::tui::Tui;
use crossterm::style::Stylize;

pub struct Reviewer;

impl Reviewer {
    pub async fn run(
        tui: &Tui,
        ai: &GeminiClient,
        repo: &GitRepo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let diff = repo.get_staged_diff()?;
        if diff.trim().is_empty() {
            println!("No staged changes to review.");
            return Ok(());
        }

        let spinner = tui.start_thinking("AI is reviewing your code for bugs & security flaws...");
        
        let system_prompt = 
            "You are a Senior Software Engineer doing a Code Review. \
             Analyze the provided git diff. \
             Focus on: \
             1. Logic Bugs \
             2. Security Vulnerabilities \
             3. Performance Issues \
             4. Idiomatic Code/Best Practices \
             \
             Output a concise list of issues in Markdown format. \
             Use emojis to indicate severity: \
             🔴 (Critical), 🟡 (Warning), 🟢 (Nitpick). \
             If the code looks good, say '✅ No issues found.'";
             
        let prompt = format!("Code to review:\n\n{}", diff);
        
        let review_report = ai.generate_text(&prompt, system_prompt).await?;
        tui.stop_spinner(spinner);
        
        tui.print_header("CODE REVIEW REPORT");
        println!("{}", review_report);
        println!("\n{}", "─".repeat(60).green());
        
        Ok(())
    }
}
