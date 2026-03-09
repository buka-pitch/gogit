use crate::theme::MarkdownRenderer;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub enum Action {
    Confirm(String),
    Reject,
    Regenerate,
    Edit,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Tui {
    pub spinner: Option<ProgressBar>,
}

#[allow(dead_code)]
impl Tui {
    pub fn new() -> Self {
        Self { spinner: None }
    }

    pub fn start_spinner(&self, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        let style = ProgressStyle::default_spinner()
            .tick_chars("▖▗▘▙▚▛▜▝▞▟")
            .template("{spinner:.green} {msg:.cyan.bold}");
        if let Ok(s) = style {
            pb.set_style(s);
        }
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn start_thinking(&self, msg: &str) -> ProgressBar {
        self.start_spinner(msg)
    }

    pub fn stop_spinner(&self, pb: ProgressBar) {
        pb.finish_and_clear();
    }

    pub fn print_token(&self, token: &str) -> io::Result<()> {
        print!("{}", Colorize::green(token).bold());
        std::io::stdout().flush()?;
        Ok(())
    }

    pub fn print_header(&self, title: &str) {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );
        println!("  {}", title.bold().cyan());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );
    }

    pub fn print_box(&self, title: &str, content: &str) {
        let width = 60;

        println!("\n┌{}┐", "─".repeat(width));
        if !title.is_empty() {
            println!("│ {} ", title.bold().cyan());
            println!("├{}┤", "─".repeat(width));
        }

        for line in content.lines() {
            println!("│ {}", line);
        }

        println!("└{}┘", "─".repeat(width));
    }

    pub fn print_markdown(&self, md: &str) {
        let rendered = MarkdownRenderer::render(md);
        if rendered.trim().is_empty() {
            println!("\n{}", md);
        } else {
            println!("\n{}", rendered);
        }
        std::io::stdout().flush().ok();
    }

    pub fn print_with_typewriter(&self, text: &str, delay_ms: u64) {
        let chars: Vec<char> = text.chars().collect();
        let mut stdout = std::io::stdout();

        for c in chars {
            if c == '\n' {
                print!("\n");
            } else if c == '`' {
                // Toggle code styling
                print!("{}", c);
            } else if c.is_uppercase() {
                print!("{}", c);
            } else {
                print!("{}", c);
            }

            let _ = stdout.flush();
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }

    pub fn print_success(&self, msg: &str) {
        println!("\n{} {}", "✓".green(), msg.green());
    }

    pub fn print_error(&self, msg: &str) {
        println!("\n{} {}", "✖".red(), msg.red());
    }

    pub fn print_warning(&self, msg: &str) {
        println!("\n{} {}", "⚠".yellow(), msg.yellow());
    }

    pub fn print_info(&self, msg: &str) {
        println!("\n{} {}", "ℹ".cyan(), msg.cyan());
    }

    pub fn print_ai_response(&self, content: &str) {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════".cyan()
        );
        println!("  {} ", "AI Response".bold().cyan());
        println!(
            "{}",
            "═══════════════════════════════════════════════════".cyan()
        );

        let rendered = MarkdownRenderer::render(content);
        println!("{}", rendered);

        println!(
            "{}",
            "═══════════════════════════════════════════════════".cyan()
        );
    }

    pub fn print_tool_call(&self, name: &str, args: &str) {
        println!(
            "\n{} Using tool: {} with args: {}",
            "🔧".cyan(),
            name.bold().yellow(),
            args.dimmed()
        );
    }

    pub fn print_tool_result(&self, result: &str) {
        let truncated = if result.len() > 200 {
            format!("{}... (total {} chars)", &result[..200], result.len())
        } else {
            result.to_string()
        };

        println!("{} Result: {}", "✔".green(), truncated.dimmed());
    }

    pub fn prompt_review(&self, current_msg: &str) -> io::Result<Action> {
        println!("\n");
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );
        println!("{}", "│ REVIEW PROPOSED COMMIT".bold().cyan());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );

        for line in current_msg.lines().take(10) {
            println!("{} {}", "│".green(), line);
        }

        if current_msg.lines().count() > 10 {
            println!(
                "{} ... ({} more lines)",
                "│".green(),
                current_msg.lines().count() - 10
            );
        }

        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );

        println!(
            "{} [Enter] {}  [e] {}  [r] {}  [Esc] {}",
            "".green().bold(),
            "Confirm".green(),
            "Edit".yellow(),
            "Regenerate".blue(),
            "Abort".red()
        );

        crossterm::terminal::enable_raw_mode()?;

        use crossterm::event::{self, Event, KeyCode};

        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Enter => {
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(Action::Confirm(current_msg.to_string()));
                        }
                        KeyCode::Char('e') => {
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(Action::Edit);
                        }
                        KeyCode::Char('r') => {
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(Action::Regenerate);
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(Action::Reject);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn prompt_multiselect(
        &self,
        items: &[String],
        prompt_text: &str,
    ) -> io::Result<Vec<String>> {
        use dialoguer::{theme::ColorfulTheme, MultiSelect};

        let theme = ColorfulTheme::default();

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt(prompt_text.bold().cyan().to_string())
            .items(items)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(selections.into_iter().map(|i| items[i].clone()).collect())
    }

    pub fn prompt_yes_no(&self, prompt_text: &str) -> io::Result<bool> {
        use dialoguer::{theme::ColorfulTheme, Confirm};

        let theme = ColorfulTheme::default();

        Confirm::with_theme(&theme)
            .with_prompt(prompt_text.bold().cyan().to_string())
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn open_editor(&self, initial_msg: &str) -> io::Result<String> {
        let edited = edit::edit(initial_msg)?;
        Ok(edited)
    }

    pub fn separator(&self) {
        println!(
            "{}",
            "═══════════════════════════════════════════════════".cyan()
        );
    }

    pub fn separator_green(&self) {
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .to_string()
                .green()
        );
    }
}
