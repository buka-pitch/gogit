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
mod agent;
mod tools;
mod assets;
mod testgen;
mod cybersec;
mod theme;

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
    /// Generate an AI commit message for staged changes
    Commit {
        /// (Hidden) Arguments passed by git hook (commit_msg_file, source, sha1)
        #[arg(hide = true)]
        args: Vec<String>,
    },
    /// Create a comprehensive AI pull request description
    Pr {
        /// Base branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// AI-driven code review of your staged changes
    Review,
    /// Precision AI refactoring based on your instructions
    Fix {
        /// File to modify
        file: String,
        /// Instructions for the refactoring
        instruction: String,
    },
    /// Semantic, natural language search through git history
    Search {
        /// Natural language search query
        query: String,
    },
    /// Intelligent project documentation management (README, etc.)
    Doc,
    /// Dry-run merge check with optional AI conflict resolution
    Check {
        /// Base branch to check against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// Translate natural language into native Git aliases
    Alias {
        /// Description of the desired git command
        description: Option<String>,
    },
    /// Create a new branch with an AI-suggested name
    Branch {
        /// Description of the work to be done in the new branch
        description: String,
    },
    /// Generate professional release notes since the last tag
    Release {
        /// Starting tag or reference (default: latest tag)
        #[arg(long)]
        from: Option<String>,
    },
    /// Ask natural language questions about your codebase
    Explain {
        /// Your question about the repository
        question: Option<String>,
    },
    /// Intelligent analysis of stale or redundant branches
    Stale,
    /// List and select from available free AI models
    Models,
    /// Start an interactive AI agent chat with tool support
    Chat {
        /// Initial query for the agent
        query: Option<String>,
    },
    /// Install or uninstall automated Git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// AI-powered test generation for multiple languages
    Testgen {
        /// Source file to generate tests for
        file: String,
        /// Test framework (jest, pytest, tokio, etc.)
        #[arg(long)]
        framework: Option<String>,
        /// Test type: unit or integration
        #[arg(long, default_value = "unit")]
        test_type: String,
        /// Output path for tests
        #[arg(long)]
        output: Option<String>,
        /// Preview only, don't save
        #[arg(long)]
        preview: bool,
    },
    /// Preview AI changes as diff before applying
    Preview {
        /// File to edit
        file: String,
        /// Instructions for changes
        #[arg(long)]
        instruction: String,
    },
    /// AI-powered code scaffolding
    Scaffold {
        /// Name of the component/file to create
        name: String,
        /// Type: component, module, class, function, service, etc.
        #[arg(long)]
        template_type: Option<String>,
        /// Output path where to save
        #[arg(long)]
        output: Option<String>,
    },
    /// Cybersecurity mode - Learn, Recon, Analyze, CTF
    Sec {
        /// Task or topic
        task: Option<String>,
        /// Mode: learn, recon, analyze, ctf, interactive
        #[arg(long)]
        mode: Option<String>,
        /// Target for recon/analyze
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Subcommand)]
enum HookAction {
    /// Install the Git hook
    Install,
    /// Uninstall the Git hook
    Uninstall,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Detect and configure Ollama automatically
    DetectOllama,
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

    if let Some(cmd) = cli.command {
        if let Err(e) = run(cmd).await {
            eprintln!("\n{} {}", "✖ Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

async fn show_main_menu() {
    let _tui = tui::Tui::new();
    
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");
    
    render_logo().await;
    println!();
    println!("{}", "  ═══════════════════════════════════════════════  ".black().on_green());
    println!("{}", "│  AI-POWERED GIT COMPANION                   │".green().bold());
    println!("{}", "  ═══════════════════════════════════════════════  ".black().on_green());
    println!();

    let choices = vec![
        "▸ AI Commit       │ Generate & commit smart messages",
        "▸ Create PR       │ Effortless pull request creation",
        "▸ Code Review    │ Find bugs and security issues",
        "▸ AI Fix         │ Let AI refactor or fix files",
        "▸ Search History │ Natural language commit search",
        "▸ Generate Doc   │ Update your project README",
        "▸ Check Conflicts│ Dry-run merge check",
        "▸ Smart Branch   │ AI-suggested branch name",
        "▸ NL Alias       │ Translate English to Git command",
        "▸ Release Notes  │ AI Categorized changelog",
        "▸ Repo Navigator│ AI answers about the code",
        "▸ Stale Branches │ Intelligent branch cleanup",
        "▸ Choose Model  │ Browse free AI models",
        "▸ AI Test Gen   │ Generate unit tests (6 languages)",
        "▸ AI Agent Chat │ Chat with tools (web, files)",
        "▸ CyberSec Mode │ Security learning & recon",
        "▸ Install Hook  │ Auto-run on every commit",
        "▸ Remove Hook   │ Restore native git behavior",
        "🚪 Exit",
    ];

    use dialoguer::{Select, theme::ColorfulTheme};
    let theme = ColorfulTheme {
        active_item_style: dialoguer::console::Style::new().green().bold(),
        inactive_item_style: dialoguer::console::Style::new().dim(),
        prompt_style: dialoguer::console::Style::new().cyan().bold(),
        ..ColorfulTheme::default()
    };

    let selection = match Select::with_theme(&theme)
        .with_prompt("Choose your next action:")
        .default(0)
        .items(&choices)
        .interact_opt() {
            Ok(Some(idx)) => idx,
            _ => 18, // Default to Exit on error
        };

    match selection {
        0 => run_wrapper(Commands::Commit { args: vec![] }).await,
        1 => run_wrapper(Commands::Pr { base: "main".to_string() }).await,
        2 => run_wrapper(Commands::Review).await,
        3 => {
            if let Ok(file) = dialoguer::Input::new().with_prompt("File to fix").interact_text() {
                if let Ok(instr) = dialoguer::Input::new().with_prompt("Instruction").interact_text() {
                    run_wrapper(Commands::Fix { file, instruction: instr }).await;
                }
            }
        },
        4 => {
             if let Ok(q) = dialoguer::Input::new().with_prompt("Search query").interact_text() {
                 run_wrapper(Commands::Search { query: q }).await;
             }
        },
        5 => run_wrapper(Commands::Doc).await,
        6 => {
            if let Ok(base) = dialoguer::Input::new()
                .with_prompt("Base branch to check against")
                .default("main".to_string())
                .interact_text()
            {
                run_wrapper(Commands::Check { base }).await;
            }
        },
        7 => {
            if let Ok(desc) = dialoguer::Input::new()
                .with_prompt("What are you working on?")
                .interact_text()
            {
                run_wrapper(Commands::Branch { description: desc }).await;
            }
        },
        8 => {
            if let Ok(desc) = dialoguer::Input::new()
                .with_prompt("Describe the git command you want")
                .interact_text()
            {
                run_wrapper(Commands::Alias { description: Some(desc) }).await;
            }
        },
        9 => run_wrapper(Commands::Release { from: None }).await,
        10 => {
            if let Ok(q) = dialoguer::Input::new()
                .with_prompt("What would you like to know about the code?")
                .interact_text()
            {
                run_wrapper(Commands::Explain { question: Some(q) }).await;
            }
        },
        11 => run_wrapper(Commands::Stale).await,
        12 => run_wrapper(Commands::Models).await,
        13 => {
            if let Ok(file) = dialoguer::Input::new()
                .with_prompt("Source file to generate tests for")
                .interact_text()
            {
                run_wrapper(Commands::Testgen { 
                    file, 
                    framework: None, 
                    test_type: "unit".to_string(), 
                    output: None, 
                    preview: false 
                }).await;
            }
        },
        14 => run_wrapper(Commands::Chat { query: None }).await,
        15 => run_wrapper(Commands::Sec { task: None, mode: None, target: None }).await,
        16 => run_wrapper(Commands::Hook { action: HookAction::Install }).await,
        17 => run_wrapper(Commands::Hook { action: HookAction::Uninstall }).await,
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
    let ai = ai::AiClient::new(
        config.model.clone(), 
        &config.provider,
        config.api_key.clone(),
        config.gemini_api_key.clone(),
        config.ollama_base_url.clone(),
    )?;
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
        Commands::Check { base } => handle_check_conflicts(&repo, &ai, &tui, &base).await?,
        Commands::Branch { description } => handle_smart_branch(&repo, &ai, &tui, &description).await?,
        Commands::Alias { description } => handle_nl_alias(&ai, &tui, description).await?,
        Commands::Release { from } => handle_release_notes(&repo, &ai, &tui, from).await?,
        Commands::Explain { question } => handle_explain(&repo, &ai, &tui, question).await?,
        Commands::Stale => handle_stale_branches(&repo, &ai, &tui, &config).await?,
        Commands::Models => handle_models(&ai, &tui, &config).await?,
        Commands::Chat { query } => handle_chat(&ai, &tui, query).await?,
        Commands::Hook { action } => match action {
            HookAction::Install => hook::HookManager::install()?,
            HookAction::Uninstall => hook::HookManager::uninstall()?,
        },
        Commands::Config { action } => handle_config(&config, action).await?,
            Commands::Testgen { file, framework, test_type, output, preview } => {
            testgen::TestGenerator::run(
                &tui,
                &ai,
                &file,
                testgen::TestOptions {
                    test_type,
                    framework,
                    output_path: output,
                    preview_only: preview,
                },
            ).await?
        },
        Commands::Preview { file, instruction } => {
            handle_preview(&tui, &ai, &file, &instruction).await?
        },
        Commands::Scaffold { name, template_type, output } => {
            handle_scaffold(&tui, &ai, &name, template_type.as_deref(), output.as_deref()).await?
        },
        Commands::Sec { task, mode, target } => {
            handle_cybersec(&tui, &ai, task, mode.as_deref(), target.as_deref()).await?
        },
    }
    Ok(())
}

async fn handle_commit(
    repo: &git::GitRepo,
    ai: &ai::AiClient,
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
    ai: &ai::AiClient,
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
         Structure: \n# [A concise, descriptive PR title]\n\n## Summary\n(One paragraph summary)\n\n## Key Changes\n- (High level bullet)\n\n## Checklist\n- [x] (Max 5 technical milestones)"
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
            
            // Extract title from the first line (if it starts with #)
            let (title, body) = if clean_msg.starts_with("# ") {
                if let Some((t, b)) = clean_msg.split_once('\n') {
                    (t.trim_start_matches("# ").trim().to_string(), b.trim().to_string())
                } else {
                    (clean_msg.trim_start_matches("# ").trim().to_string(), String::new())
                }
            } else if let Some((t, b)) = clean_msg.split_once('\n') {
                (t.trim().to_string(), b.trim().to_string())
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
    ai: &ai::AiClient,
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
        println!("\n{}", "Tip: You'll need to resolve these manually or use AI Fix.".dim());

        let fix = dialoguer::Confirm::new()
            .with_prompt("Would you like to use AI to resolve these conflicts?")
            .default(true)
            .interact()?;

        if fix {
            resolve_conflicts_interactive(repo, ai, tui, base, &result.conflicted_files).await?;
        }
    } else {
        println!("\n{}", "✔ CLEAN MERGE".green().bold());
        println!("Merging {} into {} would be conflict-free.", current_branch, base);
    }
    
    Ok(())
}

async fn resolve_conflicts_interactive(
    repo: &git::GitRepo,
    ai: &ai::AiClient,
    tui: &tui::Tui,
    base: &str,
    conflicted_files: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{} Attempting to merge {} into your current branch...", "ℹ".blue(), base);
    
    // Actually run git merge
    let mut merge_cmd = std::process::Command::new("git");
    merge_cmd.arg("merge").arg(base);
    
    let merge_output = merge_cmd.output()?;
    if merge_output.status.success() {
        println!("{} Merge completed successfully (no conflicts after all?)", "✔".green());
        return Ok(());
    }

    println!("{} Commencing AI Resolution for {} files...", "🚀".cyan(), conflicted_files.len());

    for file in conflicted_files {
        let spinner = tui.start_thinking(&format!("Resolving {}...", file));
        
        let content = std::fs::read_to_string(file)?;
        let resolved = ai.resolve_conflicts(&content, file).await?;
        
        std::fs::write(file, resolved)?;
        
        // Add resolved file
        repo.stage_files(&[file.clone()])?;
        
        tui.stop_spinner(spinner);
        println!("  {} Resolved {}", "✔".green(), file.clone().yellow());
    }

    println!("\n{} All conflicts resolved by AI!", "✨".green().bold());
    
    let commit = dialoguer::Confirm::new()
        .with_prompt("Would you like to commit the resolution?")
        .default(true)
        .interact()?;

    if commit {
        repo.commit("chore: resolve merge conflicts using gogit AI")?;
        println!("{} Resolution committed!", "✔".green());
    } else {
        println!("{} Files staged. You can review and commit manually.", "ℹ".blue());
    }

    Ok(())
}

async fn handle_nl_alias(
    ai: &ai::AiClient,
    tui: &tui::Tui,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let desc = match description {
        Some(d) => d,
        None => dialoguer::Input::new().with_prompt("Describe the git command").interact_text()?,
    };

    let spinner = tui.start_thinking("Translating to Git command...");
    let command = ai.generate_git_command(&desc).await?;
    tui.stop_spinner(spinner);

    println!("\n{} Recommended command:", "💡".yellow());
    println!("  {}", command.clone().cyan().bold());

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Would you like to save this as a git alias?")
        .default(false)
        .interact()?;

    if confirm {
        let alias_name: String = dialoguer::Input::new()
            .with_prompt("Alias name (e.g., 'recent')")
            .interact_text()?;
        
        let mut cmd = std::process::Command::new("git");
        cmd.arg("config")
           .arg("--global")
           .arg(format!("alias.{}", alias_name))
           .arg(command.trim_start_matches("git ").trim());

        let output = cmd.output()?;
        if output.status.success() {
            println!("{} Alias '{}' saved successfully!", "✔".green(), alias_name);
            println!("You can now run it using: {} {}", "git".dim(), alias_name.yellow());
        } else {
            eprintln!("{} Failed to save alias: {}", "✖".red(), String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

async fn handle_smart_branch(
    _repo: &git::GitRepo,
    ai: &ai::AiClient,
    tui: &tui::Tui,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let spinner = tui.start_thinking("Suggesting branch name...");
    let suggest_name = ai.suggest_branch_name(description).await?;
    tui.stop_spinner(spinner);

    println!("\n{} AI Suggestion: {}", "🌱".green(), suggest_name.clone().yellow().bold());

    let branch_name: String = dialoguer::Input::new()
        .with_prompt("Confirm or edit branch name")
        .default(suggest_name)
        .interact_text()?;

    let mut cmd = std::process::Command::new("git");
    cmd.arg("checkout").arg("-b").arg(&branch_name);

    let output = cmd.output()?;
    if output.status.success() {
        println!("{} Switched to a new branch '{}'", "✔".green(), branch_name);
    } else {
        eprintln!("{} Failed to create branch: {}", "✖".red(), String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn handle_release_notes(
    repo: &git::GitRepo,
    ai: &ai::AiClient,
    tui: &tui::Tui,
    from_ref: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_ref = match from_ref {
        Some(r) => Some(r),
        None => repo.get_last_tag()?,
    };

    let spinner = tui.start_thinking(&format!("Analyzing commits since {}...", base_ref.as_deref().unwrap_or("the beginning")));
    let commits = repo.get_commits_since_ref(base_ref.as_deref())?;
    tui.stop_spinner(spinner);

    if commits.trim().is_empty() {
        println!("{} No new commits found since {}", "ℹ".blue(), base_ref.unwrap_or_else(|| "the beginning".to_string()));
        return Ok(());
    }

    let spinner = tui.start_thinking("Generating RELEASE_NOTES.md...");
    let notes = ai.generate_release_notes(&commits).await?;
    tui.stop_spinner(spinner);

    println!("\n{}\n", "--- PREVIEW ---".dim());
    println!("{}", notes);
    println!("\n{}\n", "---------------".dim());

    if tui.prompt_yes_no("Save to RELEASE_NOTES.md?")? {
        std::fs::write("RELEASE_NOTES.md", &notes)?;
        println!("{} Successfully saved to RELEASE_NOTES.md", "✔".green());
    }

    Ok(())
}

async fn handle_explain(
    repo: &git::GitRepo,
    ai: &ai::AiClient,
    tui: &tui::Tui,
    question: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let q = match question {
        Some(s) => s,
        None => dialoguer::Input::new().with_prompt("What would you like to know about this repository?").interact_text()?,
    };

    let spinner = tui.start_thinking("Scanning repository structure...");
    let tree = repo.get_repo_tree()?;
    tui.stop_spinner(spinner);

    // Get some basic context: README and Cargo.toml/package.json if they exist
    let mut context_files = Vec::new();
    for f in &["README.md", "Cargo.toml", "package.json", "go.mod"] {
        if let Ok(content) = std::fs::read_to_string(f) {
            context_files.push(format!("File: {}\n---\n{}\n---", f, content));
        }
    }
    let context = context_files.join("\n\n");

    let spinner = tui.start_thinking("Architecting an answer...");
    let answer = ai.answer_repo_question(&q, &tree, &context).await?;
    tui.stop_spinner(spinner);

    println!("\n{} AI Architect Says:", "🗺️".cyan());
    println!("\n{}\n", answer);

    Ok(())
}

async fn handle_stale_branches(
    repo: &git::GitRepo,
    ai: &ai::AiClient,
    tui: &tui::Tui,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_branches = config.max_stale_branches.unwrap_or(20);
    let spinner = tui.start_thinking("Gathering branch information...");
    let branches = repo.get_branches()?;
    let current_branch = repo.get_current_branch()?;

    // We'll analyze more branches and be smarter about filtering meta-refs
    let mut branch_info = Vec::new();
    let main_branches = ["main", "master", "develop"];
    
    for b in branches.iter() {
        let b_lower = b.to_lowercase();
        
        // Skip current branch, main meta-branches, and common meta-refs like HEAD
        let is_main = main_branches.iter().any(|&m| b_lower == m || b_lower.ends_with(&format!("/{}", m)));
        if b == &current_branch || is_main || b_lower.contains("head") {
             continue;
        }
        
        if branch_info.len() >= max_branches {
            break;
        }

        let summary = repo.get_branch_summary(b, "main").unwrap_or_else(|_| "No summary available".to_string());
        branch_info.push(format!("Branch: {}\nRecent Commits:\n{}\n", b, summary));
    }
    tui.stop_spinner(spinner);

    if branch_info.is_empty() {
        println!("{} No secondary branches found to analyze.", "ℹ".blue());
        return Ok(());
    }

    let spinner = tui.start_thinking("Analyzing branch content with AI...");
    let analysis = ai.analyze_branch_staleness(&branch_info.join("\n---\n")).await?;
    tui.stop_spinner(spinner);

    println!("\n{} AI Stale Branch Analysis:", "🧹".cyan());
    println!("\n{}\n", analysis);

    println!("{} Note: No branches were deleted. Use 'git branch -d <name>' to clean up.", "ℹ".dim());

    Ok(())
}

async fn render_logo() {
    let viuer_config = viuer::Config { width: Some(80), transparent: true, ..Default::default() };

    // Primary: Try local file
    if viuer::print_from_file("img1.png", &viuer_config).is_ok() { return; }

    // Secondary: Try embedded asset via temp file
    let temp_path = std::env::temp_dir().join("gogit_logo.png");
    let _ = std::fs::write(&temp_path, assets::LOGO_BYTES);
    if viuer::print_from_file(&temp_path, &viuer_config).is_ok() { return; }

    // Tertiary: Dynamic ASCII Art
    let mut logo_vec = Vec::new();
    let ascii_options = rascii_art::RenderOptions { width: Some(80), colored: true, ..Default::default() };
    
    let cover_path = std::env::temp_dir().join("gogit_cover.png");
    let _ = std::fs::write(&cover_path, assets::COVER_BYTES);
    
    if rascii_art::render(&*cover_path.to_string_lossy(), &mut logo_vec, &ascii_options).is_ok() {
        println!("{}", String::from_utf8_lossy(&logo_vec));
        return;
    }

    // Quaternary: Static Text Art Fallback
    println!("{}", "  ________  ________  ________  ___  _________   ".green().bold());
    println!("{}", r#" |\   ____\|\   __  \|\   ____\|\  \|\___   ___\ "#.green().bold());
    println!("{}", r#" \ \  \___|\ \  \ \  \ \  \___| \  \|___ \  \_| "#.cyan().bold());
    println!("{}", r#"  \ \  \  __\ \  \ \  \ \  \  __\ \  \   \ \  \  "#.cyan().bold());
    println!("{}", r#"   \ \  \|\  \ \  \_\  \ \  \|\  \ \  \   \ \  \ "#.blue().bold());
    println!("{}", r#"    \ \_______\ \_______\ \_______\ \__\   \ \__\"#.blue().bold());
    println!("{}", r"     \|_______|\|_______|\|_______|\|__|    \|__|".magenta().bold());
}

async fn handle_config(
    config: &config::Config,
    action: ConfigAction,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::DetectOllama => {
            println!("{} Checking for Ollama...", "🔍".cyan());
            
            let available = ai::AiClient::is_ollama_available().await;
            
            if !available {
                println!("{}", "✖ Ollama is not running.".red());
                println!("\nTo use Ollama:");
                println!("  1. Install Ollama: https://ollama.com/download");
                println!("  2. Run: ollama serve");
                println!("  3. Pull a model: ollama pull llama3.2");
                return Ok(());
            }
            
            println!("{}", "✓ Ollama detected!".green());
            
            let spinner = tui::Tui::new().start_thinking("Fetching available models...");
            
            let temp_ai = ai::AiClient::new(
                "llama3.2".to_string(),
                "ollama",
                None,
                None,
                config.ollama_base_url.clone(),
            )?;
            
            let models = temp_ai.list_ollama_models().await?;
            tui::Tui::new().stop_spinner(spinner);
            
            if models.is_empty() {
                println!("{}", "✖ No Ollama models found.".red());
                println!("Run 'ollama pull <model>' to download a model.");
                return Ok(());
            }
            
            println!("\n{} Available Ollama models:", "📦".cyan());
            for (i, m) in models.iter().enumerate() {
                println!("  {}. {}", i + 1, m.name.as_deref().unwrap_or(&m.id).cyan());
            }
            
            use dialoguer::Select;
            let model_names: Vec<String> = models.iter()
                .map(|m| m.name.clone().unwrap_or(m.id.clone()))
                .collect();
            
            let selection = Select::new()
                .with_prompt("Select a model")
                .items(&model_names)
                .default(0)
                .interact()?;
            
            let selected_model = &model_names[selection];
            
            let mut config = config.clone();
            config.provider = "ollama".to_string();
            config.model = selected_model.clone();
            config.save()?;
            
            println!("\n{} Configured to use Ollama with model: {}", "✓".green().bold(), selected_model.clone().cyan());
            println!("\nYou can now use gogit commands without API keys!");
        }
    }
    
    Ok(())
}

async fn handle_models(
    ai: &ai::AiClient,
    tui: &tui::Tui,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_msg = match ai.provider {
        ai::AiProvider::Ollama => "Fetching Ollama models...",
        ai::AiProvider::Gemini => "Fetching Gemini models...",
        _ => "Fetching free models from OpenRouter...",
    };
    let spinner = tui.start_thinking(provider_msg);
    let models = ai.list_free_models().await?;
    tui.stop_spinner(spinner);

    if models.is_empty() {
        println!("\n{} No models found. Check your connection.", "✖".red());
        return Ok(());
    }

    use dialoguer::{Select, theme::ColorfulTheme};
    let theme = ColorfulTheme {
        active_item_style: dialoguer::console::Style::new().green().bold(),
        inactive_item_style: dialoguer::console::Style::new().dim(),
        prompt_style: dialoguer::console::Style::new().cyan().bold(),
        ..ColorfulTheme::default()
    };

    let mut items = Vec::new();
    items.push(format!("{} (currently using {})", "🔄 Switch Provider".yellow().bold(), config.provider.clone().cyan()));
    items.push("---------------------------------".dim().to_string());

    for m in &models {
        let name = m.name.as_deref().unwrap_or("Unknown");
        let context = m.context_length.map(|c| format!("{}k", c / 1024)).unwrap_or_else(|| "?".to_string());
        items.push(format!("{} ({}) - {}", name, context.cyan(), m.id.clone().dim()));
    }

    let selection = Select::with_theme(&theme)
        .with_prompt("Select a free AI model to use:")
        .default(0)
        .items(&items)
        .interact_opt()?;

    if let Some(index) = selection {
        if index == 0 {
            // Switch Provider
            let providers = vec!["OpenRouter", "Native Gemini"];
            let p_index = Select::with_theme(&theme)
                .with_prompt("Choose AI Provider:")
                .default(if config.provider == "openrouter" { 0 } else { 1 })
                .items(&providers)
                .interact()?;
            
            let mut new_config = config.clone();
            new_config.provider = if p_index == 0 { "openrouter".to_string() } else { "gemini".to_string() };
            
            // Set a sensible default model for the new provider
            if new_config.provider == "gemini" {
                new_config.model = "gemini-2.0-flash-exp".to_string();
            } else {
                new_config.model = "openai/gpt-4o-mini".to_string();
            }
            
            new_config.save()?;
            println!("\n{} Provider switched to: {}", "✔".green().bold(), if p_index == 0 { "OpenRouter" } else { "Gemini" }.cyan());
            return Ok(());
        }

        if index <= 1 { return Ok(()); } // Header or separator

        let selected = &models[index - 2];
        let mut new_config = config.clone();
        new_config.model = selected.id.clone();
        new_config.save()?;
        
        println!("\n{} Model updated to: {}", "✔".green().bold(), selected.id.clone().cyan());
        println!("{}", "This change has been saved to your config.toml".dim());
    }

    Ok(())
}

async fn handle_chat(
    ai: &ai::AiClient,
    tui: &tui::Tui,
    initial_query: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let system_prompt = r#"You are an AI-powered git companion and coding agent.

You have tools to:
- read_file, write_file, list_directory, get_file_tree - File operations
- run_command, execute_code - Execute commands
- web_search, read_url - Search the web
- git_status, git_diff, git_commit, etc. - Git operations
- detect_project_type, get Project analysis_workspace_info -
- generate_tests - Generate unit tests

IMPORTANT INSTRUCTIONS:
1. EXECUTE COMMANDS YOURSELF - Use run_command tool when user asks to do something that requires a command
2. NEVER tell users to "run this command in your terminal" - Do it yourself using tools
3. ANALYZE OUTPUT - After running commands, read the output and continue helping
4. ITERATE - If a command fails, analyze the error and try to fix it
5. EXPAND PATHS - Use ~/ to refer to home directory, paths like ~/Downloads work

Examples:
- User: "install docker" → Use run_command to install, analyze output, continue
- User: "check my code" → Use read_file to read it, analyze it, provide feedback
- User: "run tests" → Use run_command to run the tests

Be concise, helpful, and proactive. When you run a command, explain what you're doing but DON'T ask the user to run it."#;
    
    let mut agent = agent::Agent::new(ai.clone(), tui.clone(), system_prompt);

    if let Some(query) = initial_query {
        agent.chat(&query).await?;
    }

    loop {
        let input: String = dialoguer::Input::new()
            .with_prompt("You")
            .interact_text()?;

        if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
            break;
        }

        if input.trim().is_empty() {
            continue;
        }

        agent.chat(&input).await?;
    }

    Ok(())
}

async fn handle_preview(
    tui: &tui::Tui,
    ai: &ai::AiClient,
    file_path: &str,
    instruction: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let mut current_instruction = instruction.to_string();
    let mut content = fs::read_to_string(path)?;

    loop {
        let spinner = tui.start_thinking("Generating changes...");
        
        let system_prompt = "You are an expert code refactoring tool. \
            Show the exact changes you would make in unified diff format. \
            Output ONLY the diff - no explanations, no markdown code blocks.";

        let prompt = format!(
            "Make the following changes to {}:\n\n{}\n\n\
            ORIGINAL FILE:\n\
            ```\n{}\n```",
            file_path,
            current_instruction,
            content
        );

        let result = ai.generate_text(&prompt, system_prompt).await?;
        tui.stop_spinner(spinner);

        println!("\n{}", "═".repeat(60).cyan());
        println!("{} Preview Changes for: {}", "📝".cyan(), file_path.yellow());
        println!("{}", "═".repeat(60).cyan());
        println!("\nInstruction: {}\n", current_instruction.clone().dim());
        println!("{}", result);
        println!("\n{}", "═".repeat(60).cyan());

        let choices = vec![
            "Apply changes",
            "Edit instruction",
            "Cancel",
        ];

        let selection = dialoguer::Select::new()
            .with_prompt("Choose action:")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                let spinner = tui.start_thinking("Applying changes...");
                let apply_prompt = format!(
                    "Apply these changes to the file. Output ONLY the complete modified file content.\n\n\
                    INSTRUCTION: {}\n\n\
                    ORIGINAL:\n\
                    ```\n{}\n```",
                    current_instruction,
                    content
                );
                
                let new_content = ai.generate_text(&apply_prompt, "You are a code refactoring tool. Output ONLY the raw file content - no markdown, no explanations.").await?;
                
                let clean_new_content = if new_content.trim().starts_with("```") {
                    new_content.lines()
                        .skip(1)
                        .filter(|l| !l.trim().starts_with("```"))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    new_content
                };

                fs::write(path, clean_new_content)?;
                tui.stop_spinner(spinner);
                
                println!("\n{} Changes applied to {}", "✔".green(), file_path.yellow());
                break;
            }
            1 => {
                let new_instruction: String = dialoguer::Input::new()
                    .with_prompt("New instruction:")
                    .default(current_instruction)
                    .interact_text()?;
                
                current_instruction = new_instruction;
                // Reload content in case it changed
                content = fs::read_to_string(path)?;
            }
            _ => {
                println!("\n{} Cancelled", "ℹ".blue());
                break;
            }
        }
    }

    Ok(())
}

async fn handle_scaffold(
    tui: &tui::Tui,
    ai: &ai::AiClient,
    name: &str,
    template_type: Option<&str>,
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    let template = template_type.unwrap_or("component");
    let default_output = format!("{}.txt", name);
    let output = output_path.unwrap_or(&default_output);

    let path = Path::new(output);
    
    // Get project context
    let project_type = testgen::TestGenerator::detect_language(output);
    let project_path = path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string();
    let framework = testgen::TestGenerator::detect_framework(&project_type, &project_path);

    println!("\n{} Generating scaffold...", "⏳".cyan());
    println!("{} Name: {}", "📛".cyan(), name.yellow());
    println!("{} Type: {}", "🏷️".cyan(), template.yellow());
    println!("{} Output: {}", "📁".cyan(), output.yellow());
    println!("{} Project: {} ({})", "🗂️".cyan(), project_type.display_name(), framework.display_name());

    let spinner = tui.start_thinking("AI is generating code...");

    let system_prompt = format!(
        "You are an expert code generator. Generate {} boilerplate code for the given name. \
        Analyze the project context and generate idiomatic, production-ready code. \
        Output ONLY the code - no explanations, no markdown code blocks.",
        template
    );

    let prompt = format!(
        "Generate a {} called '{}' for a {} project using {}.\n\n\
        Project type: {} ({})\n\
        Output file: {}",
        template,
        name,
        project_type.display_name(),
        framework.display_name(),
        project_type.display_name(),
        framework.display_name(),
        output
    );

    let result = ai.generate_text(&prompt, &system_prompt).await?;
    tui.stop_spinner(spinner);

    println!("\n{}", "═".repeat(60).cyan());
    println!("{} Preview: {} ({})", "📝".cyan(), name.yellow(), template.yellow());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", result);
    println!("\n{}", "═".repeat(60).cyan());

    let choices = vec![
        "Save to file",
        "Preview only",
        "Cancel",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Choose action:")
        .items(&choices)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Clean up any markdown code blocks
            let clean_content = if result.trim().starts_with("```") {
                result.lines()
                    .skip(1)
                    .filter(|l| !l.trim().starts_with("```"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                result
            };

            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, clean_content)?;
            println!("\n{} Saved to {}", "✔".green(), output.yellow());
        }
        1 => {
            println!("\n{} Preview complete", "ℹ".blue());
        }
        _ => {
            println!("\n{} Cancelled", "ℹ".blue());
        }
    }

    Ok(())
}

async fn handle_cybersec(
    tui: &tui::Tui,
    ai: &ai::AiClient,
    task: Option<String>,
    mode: Option<&str>,
    target: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {

    let mode = mode.unwrap_or("interactive");
    
    println!("{}", cybersec::CybersecMode::get_welcome_message());

    if let Some(t) = task {
        let system_prompt = cybersec::CybersecMode::get_system_prompt(mode);
        
        let mut full_task = t.clone();
        if let Some(tgt) = target {
            full_task.push_str(&format!(" Target: {}", tgt));
        }

        let spinner = tui.start_thinking("Analyzing request...");
        let response = ai.generate_text(&full_task, &system_prompt).await?;
        tui.stop_spinner(spinner);

        println!("\n{}\n", "═".repeat(60).cyan());
        tui.print_markdown(&response);
        println!("{}\n", "═".repeat(60).cyan());

        if mode == "interactive" || mode == "learn" || mode == "recon" || mode == "analyze" || mode == "ctf" {
            interactive_cybersec_loop(tui, ai, mode).await?;
        }
    } else {
        interactive_cybersec_loop(tui, ai, mode).await?;
    }

    Ok(())
}

async fn interactive_cybersec_loop(
    tui: &tui::Tui,
    ai: &ai::AiClient,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use dialoguer::Input;

    let system_prompt = cybersec::CybersecMode::get_system_prompt(mode);

    println!("\n{} Initializing cybersecurity agent with tools...", "⏳".cyan());
    println!("{} Available: web_search, read_url, security_command", "ℹ".cyan());
    println!("{} Type 'exit' to quit\n", "ℹ".cyan());

    let mut agent = agent::Agent::new(
        ai.clone(), 
        tui.clone(), 
        &system_prompt
    );

    loop {
        let input: String = Input::new()
            .with_prompt("\n🔐 Sec >".to_string().green().to_string())
            .interact_text()?;

        if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
            println!("\n{} Thank you for using Cybersecurity Mode!", "ℹ".cyan());
            break;
        }

        if input.trim().is_empty() {
            continue;
        }

        if input.to_lowercase() == "help" {
            println!("\n{}", cybersec::CybersecMode::get_welcome_message());
            continue;
        }

        if input.to_lowercase() == "topics" {
            println!("\n{} Available security topics:", "📚".cyan());
            for (key, desc) in cybersec::SecurityTopics::get_topics() {
                println!("  - {}: {}", key, desc);
            }
            continue;
        }

        if input.to_lowercase() == "tools" {
            println!("\n{} Common security tools:", "🛠️".cyan());
            for (tool, desc) in cybersec::SecurityTopics::get_tools() {
                println!("  - {}: {}", tool, desc);
            }
            continue;
        }

        // Use agent for processing (includes tool execution with user approval)
        if let Err(e) = agent.chat(&input).await {
            println!("\n{} Error: {}", "❌".red(), e);
        }
    }

    Ok(())
}


