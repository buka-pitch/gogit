mod git;
mod ai;
mod tui;
mod config;
mod github;

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use futures_util::StreamExt;
use std::process;
use tokio;

#[derive(Parser)]
#[command(name = "gogit")]
#[command(about = "High-performance Git automation with AI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a commit message for staged changes
    Commit,
    /// Generate a PR description
    Pr {
        /// Base branch to compare against (default: main)
        #[arg(short, long, default_value = "main")]
        base: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let _ = dotenvy::dotenv();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::load();
    let repo = git::GitRepo::open()?;
    let ai_client = ai::GeminiClient::new(config.model.clone()).map_err(|e| format!("Failed to init AI client: {}", e))?;
    let tui = tui::Tui::new();
    let gh_client = github::GitHub::new().ok(); // GitHub is optional

    match cli.command.unwrap_or(Commands::Commit) {
        Commands::Commit => {
            handle_commit(&repo, &ai_client, &tui, &config).await?;
        }
        Commands::Pr { base } => {
            handle_pr(&repo, &ai_client, &tui, &config, gh_client.as_ref(), &base).await?;
        }
    }

    Ok(())
}

async fn handle_commit(
    repo: &git::GitRepo,
    ai: &ai::GeminiClient,
    tui: &tui::Tui,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diff = repo.get_staged_diff()?;
    
    // Interactive Staging
    if diff.trim().is_empty() {
        println!("No staged changes.");
        let unstaged = repo.get_unstaged_files()?;
        if !unstaged.is_empty() {
             let selected = tui.prompt_multiselect(&unstaged, "Select files to stage:")?;
             if !selected.is_empty() {
                 repo.stage_files(&selected)?;
                 diff = repo.get_staged_diff()?; // refresh
             }
        }
    }
    
    if diff.trim().is_empty() {
        println!("Nothing to commit.");
        return Ok(());
    }

    let spinner = tui.start_thinking("Analyzing code patterns...");
    let processed_diff = ai.smart_diff_summary(&diff).await?;
    tui.stop_spinner(spinner);
    
    loop {
        let system_prompt = config.commit_prompt.as_deref().unwrap_or(
            "You are an expert developer. Generate a concise, conventionally formatted git commit message for the provided diff. \
             Output ONLY the commit message. \
             Thinking Level: MEDIUM (Analyze the code logic deeply before summarizing)."
        );
        
        let prompt = format!("Here is the git diff (or summary):\n\n{}", processed_diff);

        let spinner = tui.start_thinking("Generating commit message...");
        
        let mut full_msg = String::new();
        // We need to stop spinner before streaming output
        // But we want to wait for the first token? 
        // Actually, stream_completion returns the stream *after* the request starts.
        // Let's stop the spinner once we have the stream object, or better, keep it simple.
        
        let stream_result = ai.stream_completion(&prompt, system_prompt).await;
        tui.stop_spinner(spinner); // API connected
        
        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
        };
        
        tokio::pin!(stream);
        
        tui.print_header("GENERATED MESSAGE");
        while let Some(result) = stream.next().await {
            match result {
                Ok(token) => {
                    tui.print_token(&token)?;
                    full_msg.push_str(&token);
                }
                Err(e) => eprintln!("\nStream error: {}", e),
            }
        }
        
        match tui.prompt_review(&full_msg)? {
            tui::Action::Confirm(final_msg) => {
                let spinner = tui.start_thinking("Committing & Pushing...");
                repo.commit(&final_msg)?;
                repo.push()?;
                tui.stop_spinner(spinner);
                println!("{}", "\n✔ Success! Code shipped.".green().bold());
                break;
            }
            tui::Action::Edit => {
                let edited = tui.open_editor(&full_msg)?;
                let spinner = tui.start_thinking("Committing & Pushing...");
                repo.commit(&edited)?;
                repo.push()?;
                tui.stop_spinner(spinner);
                println!("{}", "\n✔ Success! Code shipped.".green().bold());
                break;
            }
            tui::Action::Regenerate => {
                println!("\nRegenerating...");
                continue;
            }
            tui::Action::Reject => {
                println!("{}", "\nAborted.".red());
                break;
            }
        }
    }

    Ok(())
}

async fn handle_pr(
    repo: &git::GitRepo,
    ai: &ai::GeminiClient,
    tui: &tui::Tui,
    config: &config::Config,
    gh_client: Option<&github::GitHub>,
    base: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let spinner = tui.start_thinking(&format!("Analyzing commits since {}...", base));
    let context = repo.get_pr_context(base)?;
    tui.stop_spinner(spinner);
    
    let system_prompt = config.pr_prompt.as_deref().unwrap_or(
        "You are an expert developer. Generate a Pull Request description in Markdown format. \
         Include a Summary, Key Changes, and a Checklist. \
         Thinking Level: HIGH."
    );
    
    let prompt = format!("Here are the commits:\n\n{}", context);

    let spinner = tui.start_thinking("Generating PR description...");

    let mut full_msg = String::new();
    let stream_result = ai.stream_completion(&prompt, system_prompt).await;
    tui.stop_spinner(spinner);
    
    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", format!("Error: {}", e).red());
            return Ok(());
        }
    };
    tokio::pin!(stream);

    print!("\n");
    while let Some(result) = stream.next().await {
        match result {
             Ok(token) => {
                tui.print_token(&token)?;
                full_msg.push_str(&token);
            }
            Err(e) => eprintln!("\nStream error: {}", e),
        }
    }
    
    if let Some(gh) = gh_client {
        println!("\n");
        if tui.prompt_yes_no("Create PR on GitHub?")? {
            let spinner = tui.start_thinking("Submitting PR...");
            let current_branch = repo.get_current_branch()?;
            let (owner, repo_name) = repo.get_remote_info()?;
            
            // Extract a title from the body? Or ask user?
            // Simple heuristic: First line is title, rest is body.
            let (title, body) = if let Some((t, b)) = full_msg.split_once('\n') {
                (t.trim().trim_start_matches("# ").to_string(), b.trim().to_string())
            } else {
                ("Automated PR".to_string(), full_msg.clone())
            };
            
            match gh.create_pr(&title, &body, &current_branch, base, &owner, &repo_name).await {
                Ok(url) => {
                    tui.stop_spinner(spinner);
                    println!("\n{} {}", "✔ PR Created:".green().bold(), url);
                }
                Err(e) => {
                    tui.stop_spinner(spinner);
                    eprintln!("\n{} {}", "✖ Failed to create PR:".red().bold(), e);
                }
            }
        }
    } else {
        println!("\n\n(GitHub token not found. Set GITHUB_TOKEN to enable auto-submission)");
    }

    Ok(())
}
