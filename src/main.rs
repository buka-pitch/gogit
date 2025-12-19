mod git;
mod ai;
mod tui;
mod config;
mod github;
mod review;
mod hook;
mod refactor;
mod search;
mod doc;

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use futures_util::StreamExt;
use std::process;
use tokio;

#[derive(Parser)]
#[command(name = "gogit")]
#[command(about = "AI-powered Git CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a commit message for staged changes
    Commit {
        /// (Hidden) Arguments passed by git hook (commit_msg_file, source, sha1)
        #[arg(hide = true)]
        args: Vec<String>,
    },
    /// Generate a PR description
    Pr {
        /// Base branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// Review code changes
    Review,
    /// Fix a specific issue in a file
    Fix {
        file: String,
        instruction: String,
    },
    /// Semantic search in git history
    Search {
        query: String,
    },
    /// Generate project documentation
    Doc,
    /// Manage git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand)]
enum HookAction {
    /// Install the Git hook
    Install,
    /// Uninstall the Git hook
    Uninstall,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Load .env
    let _ = dotenvy::dotenv();

    // Check for "interactive mode" (no args)
    if let None = cli.command {
        show_main_menu().await;
        return;
    }

    if let Err(e) = run(cli.command.unwrap()).await {
        eprintln!("\n{} {}", "✖ Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn show_main_menu() {
    let _tui = tui::Tui::new();
    
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");
    
    // BIG ASCII Art Logo (Centered)
    println!("{}", r#"
      ____  ____  ____  ___ ____ 
     / ___|/ _ \| ___||_ _|_   _|
    | |  _| | | | |  _ | |  | |  
    | |_| | |_| | |_| || |  | |  
     \____|\___/ \____|___| |_|   
    "#.green().bold());
    println!("{}", "           AI-POWERED GIT COMPANION           ".black().on_green());
    println!();

    let choices = vec![
        "✨ AI Commit     (Generate & Commit)",
        "🚀 Create PR     (Draft & Support)",
        "🕵️  Code Review   (Find Bugs & Issues)",
        "🛠️  AI Fix        (Refactor/Fix File)",
        "🔍 Search History (Natural Language Search)",
        "📑 Generate Doc   (Update README.md)",
        "🪝  Install Hook  (Auto-run on git commit)",
        "🗑️  Remove Hook   (Restore native git)",
        "🚪 Exit",
    ];

    use dialoguer::{Select, theme::ColorfulTheme};
    let theme = ColorfulTheme {
        active_item_style: dialoguer::console::Style::new().green().bold(),
        ..ColorfulTheme::default()
    };

    let selection = Select::with_theme(&theme)
        .with_prompt("Select an operation:")
        .default(0)
        .items(&choices)
        .interact()
        .unwrap_or(5); // Default to Exit on error

    match selection {
        0 => run_wrapper(Commands::Commit { args: vec![] }).await,
        1 => run_wrapper(Commands::Pr { base: "main".to_string() }).await,
        2 => run_wrapper(Commands::Review).await,
        3 => {
            let file: String = dialoguer::Input::new().with_prompt("File to fix").interact_text().unwrap();
            let instr: String = dialoguer::Input::new().with_prompt("Instruction").interact_text().unwrap();
            run_wrapper(Commands::Fix { file, instruction: instr }).await;
        },
        4 => {
             let q: String = dialoguer::Input::new().with_prompt("Search query").interact_text().unwrap();
             run_wrapper(Commands::Search { query: q }).await;
        },
        5 => run_wrapper(Commands::Doc).await,
        6 => run_wrapper(Commands::Hook { action: HookAction::Install }).await,
        7 => run_wrapper(Commands::Hook { action: HookAction::Uninstall }).await,
        _ => println!("{}", "Bye!".cyan()),
    }
}

async fn run_wrapper(cmd: Commands) {
    if let Err(e) = run(cmd).await {
         eprintln!("\n{} {}", "✖ Error:".red().bold(), e);
         process::exit(1);
    }
}

async fn run(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::load();
    let repo = git::GitRepo::open()?;
    let ai = ai::GeminiClient::new(config.model.clone())?;
    let tui = tui::Tui::new();
    
    let github = github::GitHub::new().ok();

    match command {
        Commands::Commit { args } => {
            let hook_args = if args.is_empty() { None } else { Some(args) };
            handle_commit(&repo, &ai, &tui, &config, hook_args).await?;
        },
        Commands::Pr { base } => handle_pr(&repo, &ai, &tui, &config, github.as_ref(), &base).await?,
        Commands::Review => review::Reviewer::run(&tui, &ai, &repo).await?,
        Commands::Fix { file, instruction } => refactor::Refactorer::run(&tui, &ai, &file, &instruction).await?,
        Commands::Search { query } => search::HistorySearcher::run(&tui, &ai, &repo, &query).await?,
        Commands::Doc => doc::DocGenerator::run_menu(&tui, &ai).await?,
        Commands::Hook { action } => match action {
            HookAction::Install => hook::HookManager::install()?,
            HookAction::Uninstall => hook::HookManager::uninstall()?,
        },
    }
    Ok(())
}

async fn handle_commit(
    repo: &git::GitRepo,
    ai: &ai::GeminiClient,
    tui: &tui::Tui,
    config: &config::Config,
    hook_args: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diff = repo.get_staged_diff()?;
    
    // Interactive Staging (Skip in Hook Mode to avoid locking issues if git is holding index)
    // Actually, prepare-commit-msg runs after index is locked for commit?
    // Usually it's safe to read, but modifying index might be tricky.
    // Let's allow staging in Standalone mode only.
    if hook_args.is_none() {
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
    } else {
         // In hook mode, if diff is empty, we must abort or user sees nothing?
         // Git usually prevents commit if empty, unless --allow-empty.
         if diff.trim().is_empty() {
             // For now, let's assume git handles the empty check, or we just generate nothing.
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
        
        // Strip Markdown code blocks if present
        let clean_msg = if full_msg.trim().starts_with("```") {
            full_msg.lines()
                .skip(1) // Skip first line (``` or ```markdown)
                .filter(|l| !l.trim().starts_with("```")) // Skip closing ```
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string()
        } else {
            full_msg.trim().to_string()
        };
        
        match tui.prompt_review(&clean_msg)? {
            tui::Action::Confirm(final_msg) => {
                if let Some(args) = &hook_args {
                    // Hook Mode: Write to file
                    if let Some(filepath) = args.get(0) {
                        std::fs::write(filepath, final_msg)?;
                        println!("{}", "\n✔ Message saved. Git will now commit.".green().bold());
                    } else {
                        eprintln!("Error: Hook called but no argument provided for message file.");
                    }
                    // Hook Mode does NOT push automatically, as that would happen inside the commit command? 
                    // No, push happens after. 
                    // If user wants to push, they should run 'git push' after 'git commit'.
                } else {
                    // Standalone Mode: Commit & Push
                    let spinner = tui.start_thinking("Committing & Pushing...");
                    repo.commit(&final_msg)?;
                    repo.push()?;
                    tui.stop_spinner(spinner);
                    println!("{}", "\n✔ Success! Code shipped.".green().bold());
                }
                break;
            }
            tui::Action::Edit => {
                let edited = tui.open_editor(&full_msg)?;
                if let Some(args) = &hook_args {
                     if let Some(filepath) = args.get(0) {
                        std::fs::write(filepath, edited)?;
                         println!("{}", "\n✔ Message saved. Git will now commit.".green().bold());
                    }
                } else {
                    let spinner = tui.start_thinking("Committing & Pushing...");
                    repo.commit(&edited)?;
                    repo.push()?;
                    tui.stop_spinner(spinner);
                    println!("{}", "\n✔ Success! Code shipped.".green().bold());
                }
                break;
            }
            tui::Action::Regenerate => {
                println!("\nRegenerating...");
                continue;
            }
            tui::Action::Reject => {
                println!("{}", "\nAborted.".red());
                // In hook mode, if we exit 1, git ignores the hook or aborts? 
                // prepare-commit-msg exit code > 0 aborts commit? Yes.
                if hook_args.is_some() {
                    process::exit(1);
                }
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
    
    // Strip Markdown code blocks if present
    let clean_msg = if full_msg.trim().starts_with("```") {
        full_msg.lines()
            .skip(1)
            .filter(|l| !l.trim().starts_with("```"))
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string()
    } else {
        full_msg.trim().to_string()
    };

    if let Some(gh) = gh_client {
        println!("\n");
        if tui.prompt_yes_no("Create PR on GitHub?")? {
            let spinner = tui.start_thinking("Submitting PR...");
            let current_branch = repo.get_current_branch()?;
            let (owner, repo_name) = repo.get_remote_info()?;
            
            // Extract a title from the body? Or ask user?
            // Simple heuristic: First line is title, rest is body.
            let (title, body) = if let Some((t, b)) = clean_msg.split_once('\n') {
                (t.trim().trim_start_matches("# ").to_string(), b.trim().to_string())
            } else {
                ("Automated PR".to_string(), clean_msg.clone())
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
