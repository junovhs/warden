// src/tui/config_state.rs
use crate::config::{save_to_file, Config, RuleConfig};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::collections::HashMap;
use std::time::Duration;

#[derive(PartialEq)]
pub enum ConfigField {
    MaxTokens,
    MaxComplexity,
    MaxDepth,
    MaxArgs,
    MaxWords,
}

pub struct ConfigApp {
    pub rules: RuleConfig,
    pub commands: HashMap<String, Vec<String>>,
    pub selected_field: usize,
    pub running: bool,
    pub modified: bool,
    pub saved_message: Option<(String, std::time::Instant)>,
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigApp {
    #[must_use]
    pub fn new() -> Self {
        let mut config = Config::new();
        config.load_local_config();

        Self {
            rules: config.rules,
            commands: config.commands,
            selected_field: 0,
            running: true,
            modified: false,
            saved_message: None,
        }
    }

    /// Runs the config TUI loop.
    ///
    /// # Errors
    /// Returns error if terminal IO or event polling fails.
    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<()> {
        while self.running {
            terminal.draw(|f| crate::tui::config_view::draw(f, self))?;
            self.process_event()?;
        }
        Ok(())
    }

    fn process_event(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                self.handle_input(key.code);
            }
        }
        self.check_message_expiry();
        Ok(())
    }

    fn check_message_expiry(&mut self) {
        if let Some((_, time)) = self.saved_message {
            if time.elapsed() > Duration::from_secs(2) {
                self.saved_message = None;
            }
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Left | KeyCode::Char('h') => self.decrement_val(),
            KeyCode::Right | KeyCode::Char('l') => self.increment_val(),
            KeyCode::Enter | KeyCode::Char('s') => self.save(),
            _ => {}
        }
    }

    fn move_up(&mut self) {
        if self.selected_field > 0 {
            self.selected_field -= 1;
        } else {
            self.selected_field = 4;
        }
    }

    fn move_down(&mut self) {
        if self.selected_field < 4 {
            self.selected_field += 1;
        } else {
            self.selected_field = 0;
        }
    }

    fn increment_val(&mut self) {
        self.modified = true;
        match self.selected_field {
            0 => self.rules.max_file_tokens += 100,
            1 => self.rules.max_cyclomatic_complexity += 1,
            2 => self.rules.max_nesting_depth += 1,
            3 => self.rules.max_function_args += 1,
            4 => self.rules.max_function_words += 1,
            _ => {}
        }
    }

    fn decrement_val(&mut self) {
        self.modified = true;
        match self.selected_field {
            0 => {
                if self.rules.max_file_tokens > 100 {
                    self.rules.max_file_tokens -= 100;
                }
            }
            1 => {
                if self.rules.max_cyclomatic_complexity > 1 {
                    self.rules.max_cyclomatic_complexity -= 1;
                }
            }
            2 => {
                if self.rules.max_nesting_depth > 1 {
                    self.rules.max_nesting_depth -= 1;
                }
            }
            3 => {
                if self.rules.max_function_args > 1 {
                    self.rules.max_function_args -= 1;
                }
            }
            4 => {
                if self.rules.max_function_words > 1 {
                    self.rules.max_function_words -= 1;
                }
            }
            _ => {}
        }
    }

    fn save(&mut self) {
        if let Err(e) = save_to_file(&self.rules, &self.commands) {
            self.saved_message = Some((format!("Error: {e}"), std::time::Instant::now()));
        } else {
            self.saved_message = Some(("Saved warden.toml!".to_string(), std::time::Instant::now()));
            self.modified = false;
        }
    }
}