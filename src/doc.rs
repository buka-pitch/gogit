use crate::ai::GeminiClient;
use crate::tui::Tui;
use crate::git::GitRepo;
use std::fs;
use std::path::Path;
use crossterm::style::Stylize;

pub struct DocGenerator;

impl DocGenerator {
    pub async fn run_readme(
        tui: &Tui,
        ai: &GeminiClient,
        repo: &GitRepo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let readme_path = Path::new("README.md");
        let exists = readme_path.exists();

        let mut diff = String::new();
        let mut force_general = false;

        if exists {
            let spinner = tui.start_thinking("Analyzing staged changes for README update...");
            diff = repo.get_staged_diff()?;
            tui.stop_spinner(spinner);

            if diff.is_empty() {
                println!("\n{} No staged changes found to update README with.", "ℹ".blue());
                if tui.prompt_yes_no("Perform a general project-wide update instead?")? {
                    force_general = true;
                } else {
                    return Ok(());
                }
            }
        }
        let (context, system_prompt) = if !exists || force_general {
            let spinner = tui.start_thinking("Analyzing project structure...");
            let ctx = Self::build_project_context()?;
            tui.stop_spinner(spinner);
            
            let sys = format!("You are a technical writer. Based on the provided project context, generate a professional, comprehensive README.md. \
                Include Intro, Features, Installation, and Usage. Output ONLY the raw Markdown. \
                GROUND TRUTH CLI COMMANDS: \
                - gogit commit [--force]: Generate AI commit message for staged changes. \
                - gogit pr [--base <branch>]: Generate AI PR description. \
                - gogit review: AI code review of staged changes for bugs/security. \
                - gogit fix <file> <instruction>: AI-assisted refactoring of a file. \
                - gogit search <query>: Semantic search in git history using natural language. \
                - gogit doc: Manage documentation (README updates or doc-comments). \
                - gogit hook <install|uninstall>: Install/Uninstall git hooks.");
            
            (ctx, sys)
        } else {
            // Update mode with diff
            let existing_content = fs::read_to_string(readme_path)?;
            let ctx = format!("EXISTING README:\n{}\n\nSTAGED CHANGES:\n{}", existing_content, diff);
            
            let sys = format!("You are a technical writer. An existing README.md and STAGED CHANGES are provided. Update the README to reflect these changes. \
                Integrate new features/changes without removing existing unrelated content. Output the FULL updated README.md. Output ONLY raw Markdown. \
                GROUND TRUTH CLI COMMANDS: \
                - gogit commit [--force]: Generate AI commit message for staged changes. \
                - gogit pr [--base <branch>]: Generate AI PR description. \
                - gogit review: AI code review of staged changes for bugs/security. \
                - gogit fix <file> <instruction>: AI-assisted refactoring of a file. \
                - gogit search <query>: Semantic search in git history using natural language. \
                - gogit doc: Manage documentation (README updates or doc-comments). \
                - gogit hook <install|uninstall>: Install/Uninstall git hooks.");

            (ctx, sys)
        };

        if context.is_empty() {
            return Err("Could not find any code in src/ to analyze for documentation.".into());
        }

        println!("Context size: {} characters. Sending to AI...", context.len());
        let spinner = tui.start_thinking("AI is drafting your README...");
        let new_readme = ai.generate_text(&context, &system_prompt).await?;
        tui.stop_spinner(spinner);

        let clean_readme = if new_readme.trim().starts_with("```") {
             new_readme.lines()
                .skip(1)
                .filter(|l| !l.trim().starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string()
        } else {
            new_readme.trim().to_string()
        };

        if clean_readme.is_empty() {
            return Err("Failed to parse AI response into valid Markdown.".into());
        }

        println!("\n{}", "--- PROPOSED README.md ---".yellow().bold());
        let preview: String = clean_readme.lines().take(15).collect::<Vec<&str>>().join("\n");
        println!("{}\n...", preview);

        if tui.prompt_yes_no("Save these changes to README.md?")? {
            fs::write("README.md", clean_readme)?;
            println!("\n{} README.md updated.", "✔".green().bold());
        }

        Ok(())
    }

    fn build_project_context() -> Result<String, std::io::Error> {
        let mut ctx = String::new();
        let src_dir = Path::new("src");
        if !src_dir.exists() {
            return Ok(String::from("No src directory found."));
        }

        let files = fs::read_dir(src_dir)?;
        let mut file_count = 0;
        for entry in files {
            let entry = entry?;
            let path = entry.path();
            if file_count >= 10 { break; } 
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let name = path.file_name().unwrap().to_string_lossy();
                let content = fs::read_to_string(&path)?;
                let snippet: String = content.lines().take(40).collect::<Vec<&str>>().join("\n");
                ctx.push_str(&format!("\nFILE: {}\nCONTENT SNIPPET (First 40 lines):\n{}\n", name, snippet));
                file_count += 1;
            }
        }
        Ok(ctx)
    }

    pub async fn run_comments(
        tui: &Tui,
        ai: &GeminiClient,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let spinner = tui.start_thinking(&format!("Generating comments for {}...", file_path));

        let system_prompt = "You are an expert documentation tool. \
            Add professional triple-slash (///) doc comments to all public functions and structs in the provided Rust code. \
            Include 'Arguments' and 'Returns' sections if applicable. \
            Output ONLY the raw content of the entire file after adding comments. \
            Do not include markdown code blocks. \
            Note: This tool is part of 'gogit doc', an AI-powered git companion.";

        let new_content = ai.generate_text(&content, system_prompt).await?;
        tui.stop_spinner(spinner);

        let clean_content = if new_content.trim().starts_with("```") {
             new_content.lines()
                .skip(1)
                .filter(|l| !l.trim().starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string()
        } else {
            new_content.trim().to_string()
        };

        if tui.prompt_yes_no(&format!("Apply documentation to {}?", file_path))? {
            fs::write(path, clean_content)?;
            println!("\n{} Documentation applied.", "✔".green().bold());
        }

        Ok(())
    }

    pub async fn run_menu(
        tui: &Tui,
        ai: &GeminiClient,
        repo: &GitRepo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let options = vec![
            "📝 Update README.md",
            "💡 Generate Doc-comments for a file",
        ];

        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select documentation task")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => Self::run_readme(tui, ai, repo).await?,
            1 => {
                let file: String = dialoguer::Input::new().with_prompt("File to document").interact_text()?;
                Self::run_comments(tui, ai, &file).await?;
            },
            _ => {},
        }

        Ok(())
    }
}

