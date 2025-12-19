use crate::ai::GeminiClient;
use crate::tui::Tui;
use std::fs;
use std::path::Path;
use crossterm::style::Stylize;

pub struct DocGenerator;

impl DocGenerator {
    pub async fn run_readme(
        tui: &Tui,
        ai: &GeminiClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spinner = tui.start_thinking("Analyzing project structure for Documentation...");

        // Scan src directory for a high-level view
        let files = fs::read_dir("src")?;
        let mut context = String::new();
        let mut file_count = 0;
        for entry in files {
            let entry = entry?;
            let path = entry.path();
            if file_count > 10 { break; } // Limit to top 10 files to save tokens
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let name = path.file_name().unwrap().to_string_lossy();
                let content = fs::read_to_string(&path)?;
                // Only take first 30 lines (usually imports and high-level structs)
                let snippet: String = content.lines().take(30).collect::<Vec<&str>>().join("\n");
                context.push_str(&format!("\nFILE: {}\nCONTENT SNIPPET:\n{}\n", name, snippet));
                file_count += 1;
            }
        }

        let system_prompt = "You are a technical writer. \
            Based on the provided source code snippets, generate a professional, comprehensive README.md for this project. \
            Include sections for Intro, Features, Installation, and Usage. \
            Output ONLY the raw Markdown.";

        let prompt = format!("PROJECT SOURCE CONTEXT:\n{}", context);

        let new_readme = ai.generate_text(&prompt, system_prompt).await?;
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
            new_readme.clone()
        };

        println!("\n{}", "--- GENERATED README.md ---".yellow().bold());
        println!("{}", clean_readme);

        if tui.prompt_yes_no("Save this as README.md?")? {
            fs::write("README.md", clean_readme)?;
            println!("\n{} README.md updated.", "✔".green().bold());
        }

        Ok(())
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
            Do not include markdown code blocks.";

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
            0 => Self::run_readme(tui, ai).await?,
            1 => {
                let file: String = dialoguer::Input::new().with_prompt("File to document").interact_text()?;
                Self::run_comments(tui, ai, &file).await?;
            },
            _ => {},
        }

        Ok(())
    }
}

