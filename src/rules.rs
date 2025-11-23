use crate::analysis::Analyzer;
use crate::config::Config;
use crate::error::Result;
use crate::tokens::Tokenizer;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// Thread-safe analyzer instance
static ANALYZER: LazyLock<Analyzer> = LazyLock::new(Analyzer::new);

pub struct RuleEngine {
    config: Config,
}

impl RuleEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Checks a file for violations. Returns `Ok(false)` if violations found.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read (and is not skipped).
    /// Note: Most read errors are suppressed/ignored in the current logic, returning `Ok(true)`.
    pub fn check_file(&self, path: &Path) -> Result<bool> {
        let Ok(content) = fs::read_to_string(path) else {
            return Ok(true); // Skip unreadable files
        };

        if content.contains("// warden:ignore") || content.contains("# warden:ignore") {
            return Ok(true);
        }

        let filename = path.to_string_lossy();
        let mut passed = true;

        // 1. LAW OF ATOMICITY (Token Limit)
        let token_count = Tokenizer::count(&content);
        if token_count > self.config.rules.max_file_tokens {
            Self::print_violation(
                &filename,
                0,
                &format!(
                    "File size is {token_count} tokens (Limit: {})",
                    self.config.rules.max_file_tokens
                ),
                "LAW OF ATOMICITY",
                "Split this file into smaller modules.",
            );
            passed = false;
        }

        // 2. AST ANALYSIS (Paranoia + Bluntness)
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let violations = ANALYZER.analyze(ext, &content, &self.config.rules);

            for v in violations {
                Self::print_violation(
                    &filename,
                    v.row,
                    &v.message,
                    v.law,
                    if v.law == "LAW OF BLUNTNESS" {
                        "Rename function."
                    } else {
                        "Add Error Handling."
                    },
                );
                passed = false;
            }
        }

        Ok(passed)
    }

    fn print_violation(filename: &str, row: usize, msg: &str, law: &str, help: &str) {
        let line_num = row + 1;
        println!("{}: {}", "error".red().bold(), msg.bold());
        println!("  {} {}:{}:1", "-->".blue(), filename, line_num);
        println!("   {}", "|".blue());
        println!("   {} {}: {}", "=".blue().bold(), law.white().bold(), help);
        println!();
    }
}
