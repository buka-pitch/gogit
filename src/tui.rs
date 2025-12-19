use crossterm::style::Stylize;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{stdout, Write};
use std::io;

pub enum Action {
    Confirm(String),
    Reject,
    Regenerate,
    Edit,
}

pub struct Tui {
    spinner: Option<ProgressBar>,
}

impl Tui {
    pub fn new() -> Self {
        Self { spinner: None }
    }

    pub fn start_spinner(&self, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏⚡") 
                .template("{spinner:.green} {msg:.cyan.bold}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb
    }

    pub fn start_thinking(&self, msg: &str) -> ProgressBar {
        self.start_spinner(msg)
    }

    pub fn stop_spinner(&self, pb: ProgressBar) {
        pb.finish_and_clear();
    }

    pub fn print_token(&self, token: &str) -> io::Result<()> {
        print!("{}", token.green());
        stdout().flush()?;
        Ok(())
    }

    pub fn print_header(&self, title: &str) {
        println!("\n{}", format!("=== {} ===", title).bold().on_green().black());
    }

    pub fn prompt_review(&self, current_msg: &str) -> io::Result<Action> {
        println!("\n");
        println!("{}", "─".repeat(60).green());
        println!("{}", "REVIEW PROPOSED COMMIT".bold().cyan());
        println!("{}", "─".repeat(60).green());
        
        for line in current_msg.lines() {
            println!("{} {}", "│".green(), line);
        }
        println!("{}", "─".repeat(60).green());
        
        println!(
            "{} {} {} {}", 
            "[Enter]".bold().green(), "Confirm", 
            "[e]".bold().yellow(), "Edit", 
        );
        println!(
            "{} {}  {} {}", 
            "[r]".bold().blue(), "Regenerate",
            "[Esc]".bold().red(), "Abort"
        );

        crossterm::terminal::enable_raw_mode()?;
        
        use crossterm::event::{self, Event, KeyCode};
        
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
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

    pub fn prompt_multiselect(&self, items: &[String], prompt_text: &str) -> io::Result<Vec<String>> {
        use dialoguer::{MultiSelect, theme::ColorfulTheme};
        
        let theme = ColorfulTheme {
            checked_item_prefix: dialoguer::console::Style::new().green().apply_to("✓".to_string()),
            unchecked_item_prefix: dialoguer::console::Style::new().dim().apply_to("○".to_string()),
            active_item_style: dialoguer::console::Style::new().cyan().bold(),
            ..ColorfulTheme::default()
        };

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt(prompt_text.bold().green().to_string())
            .items(items)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
        Ok(selections.into_iter().map(|i| items[i].clone()).collect())
    }

    pub fn prompt_yes_no(&self, prompt_text: &str) -> io::Result<bool> {
        use dialoguer::{Confirm, theme::ColorfulTheme};
        // Removed invalid yes_style/no_style fields
         let theme = ColorfulTheme {
            values_style: dialoguer::console::Style::new().green(),
            ..ColorfulTheme::default()
        };
        
        Confirm::with_theme(&theme)
            .with_prompt(prompt_text.bold().cyan().to_string())
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn open_editor(&self, initial_msg: &str) -> io::Result<String> {
        let edited = edit::edit(initial_msg)?;
        Ok(edited)
    }
}
