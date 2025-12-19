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
    /// Check for merge conflicts
    Check {
        /// Base branch to check against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
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
    
    // Attempt to render the actual image using viuer
    let viuer_config = viuer::Config {
        width: Some(80),
        transparent: true,
        ..Default::default()
    };

    if let Err(_) = viuer::print_from_file("img1.png", &viuer_config) {
        // Fallback 1: Dynamic ASCII Art
        let mut logo = String::new();
        let ascii_options = rascii_art::RenderOptions {
            width: Some(80),
            colored: true,
            ..Default::default()
        };

        if let Err(_) = rascii_art::render_to("img1.png", &mut logo, &ascii_options) {
            // Fallback 2: Manual ASCII Art
            println!("{}", "  ________  ________  ________  ___  _________   ".green().bold());
            println!("{}", r#" |\   ____\|\   __  \|\   ____\|\  \|\___   ___\ "#.green().bold());
            println!("{}", r#" \ \  \___|\ \  \ \  \ \  \___| \  \|___ \  \_| "#.cyan().bold());
            println!("{}", r#"  \ \  \  __\ \  \ \  \ \  \  __\ \  \   \ \  \  "#.cyan().bold());
            println!("{}", r#"   \ \  \|\  \ \  \_\  \ \  \|\  \ \  \   \ \  \ "#.blue().bold());
            println!("{}", r#"    \ \_______\ \_______\ \_______\ \__\   \ \__\"#.blue().bold());
            println!("{}", r"     \|_______|\|_______|\|_______|\|__|    \|__|".magenta().bold());
        } else {
            println!("{}", logo);
        }
    }
    println!();
    println!("{}", "             ✨ AI-POWERED GIT COMPANION ✨           ".black().on_green());
    println!();

    let choices = vec![
        "✨ AI Commit      - Generate & commit smart messages",
        "🚀 Create PR      - Effortless pull request creation",
        "🕵️ Code Review    - Find bugs and security issues",
        "🛠️ AI Fix         - Let AI refactor or fix files",
        "🔍 Search History - Natural language commit search",
        "📑 Generate Doc    - Update your project README",
        "⚔️ Check Conflicts - Dry-run merge check",
        "🪝 Install Hook   - Auto-run on every commit",
        "🗑️ Remove Hook    - Restore native git behavior",
        "🚪 Exit",
    ];

    use dialoguer::{Select, theme::ColorfulTheme};
    let theme = ColorfulTheme {
        active_item_style: dialoguer::console::Style::new().green().bold(),
        inactive_item_style: dialoguer::console::Style::new().dim(),
        prompt_style: dialoguer::console::Style::new().cyan().bold(),
        ..ColorfulTheme::default()
    };

    let selection = Select::with_theme(&theme)
        .with_prompt("Choose your next action:")
        .default(0)
        .items(&choices)
        .interact()
        .unwrap_or(9); // Default to Exit on error

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
        6 => {
            let base: String = dialoguer::Input::new()
                .with_prompt("Base branch to check against")
                .default("main".to_string())
                .interact_text()
                .unwrap();
            run_wrapper(Commands::Check { base }).await;
        },
        7 => run_wrapper(Commands::Hook { action: HookAction::Install }).await,
        8 => run_wrapper(Commands::Hook { action: HookAction::Uninstall }).await,
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
        Commands::Doc => doc::DocGenerator::run_menu(&tui, &ai, &repo).await?,
        Commands::Check { base } => handle_check_conflicts(&repo, &tui, &base).await?,
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
        "You are an expert developer. Generate a minimalist Pull Request description in Markdown format. \
         Focus on the COHESIVE PURPOSE of the changes at an executive level. \
         Do not list every single commit as a checklist item. \
         Base your description STRICTLY on the provided commits. \
         Structure: \n# Summary\n(One paragraph summary)\n\n# Key Changes\n- (High level bullet)\n\n# Checklist\n- [x] (Max 5 technical milestones)"
    );
    
    let prompt = format!(
        "STRICT INSTRUCTION: Only use the following commits to generate the PR. \
        Ignore any external context or previous knowledge. \
        COMMITS:\n\n{}", 
        context
    );

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
            
            let (title, body) = if let Some((t, b)) = clean_msg.split_once('\n') {
                (t.trim().trim_start_matches("# ").to_string(), b.trim().to_string())
            } else {
                ("Automated PR".to_string(), clean_msg.clone())
            };
            
            match gh.create_pr(&title, &body, &current_branch, base, &owner, &repo_name).await {
                Ok(url) => {
                    tui.stop_spinner(spinner);
                    println!("\n{} PR Created: {}", "✔".green().bold(), url.cyan());
                }
                Err(e) => {
                    tui.stop_spinner(spinner);
                    eprintln!("\n{} {}", "✖ Failed to create PR:".red().bold(), e);
                }
            }
        }
    } else {
        println!("\n{} GitHub token not found (env: GITHUB_TOKEN missing).", "ℹ".blue());
        if tui.prompt_yes_no("Try submitting PR via GitHub CLI ('gh pr create') instead?")? {
             let current_branch = repo.get_current_branch()?;
             let (title, body) = if let Some((t, b)) = clean_msg.split_once('\n') {
                (t.trim().trim_start_matches("# ").to_string(), b.trim().to_string())
            } else {
                ("Automated PR".to_string(), clean_msg.clone())
            };

            let spinner = tui.start_thinking("Calling GitHub CLI...");
            let output = std::process::Command::new("gh")
                .arg("pr")
                .arg("create")
                .arg("--title")
                .arg(&title)
                .arg("--body")
                .arg(&body)
                .arg("--base")
                .arg(base)
                .arg("--head")
                .arg(&current_branch)
                .output();
            
            tui.stop_spinner(spinner);

            match output {
                Ok(out) if out.status.success() => {
                    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    println!("\n{} PR Created: {}", "✔".green().bold(), url.cyan());
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    eprintln!("\n{} CLI error: {}", "✖ Failed:".red().bold(), err);
                }
                Err(e) => {
                    eprintln!("\n{} Could not run 'gh' command: {}", "✖ Error:".red().bold(), e);
                }
            }
        }
    }

    Ok(())
}

async fn handle_check_conflicts(
    repo: &git::GitRepo,
    tui: &tui::Tui,
    base: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_branch = repo.get_current_branch()?;
    
    let spinner = tui.start_thinking(&format!("Checking for conflicts between {} and {}...", base, current_branch));
    let result = repo.check_merge_conflicts(base, &current_branch)?;
    tui.stop_spinner(spinner);
    
    if result.has_conflicts {
        println!("\n{}", "✖ MERGE CONFLICTS DETECTED".red().bold());
        println!("Merging {} into {} would result in conflicts in the following files:", current_branch, base);
        for file in &result.conflicted_files {
            println!("  - {}", file.clone().yellow());
        }
        println!("\n{}", "Tip: You'll need to resolve these manually before pushing.".dim());
    } else {
        println!("\n{}", "✔ CLEAN MERGE".green().bold());
        println!("Merging {} into {} would be conflict-free.", current_branch, base);
    }
    
    Ok(())
}
