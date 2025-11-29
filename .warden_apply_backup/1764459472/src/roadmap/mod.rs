pub mod cli;
pub mod cmd_parser;
pub mod cmd_runner;
pub mod display;
pub mod parser;
pub mod prompt;
pub mod types;

// Re-export CommandBatch from types, not cmd_parser
pub use cmd_runner::apply_commands;
pub use parser::slugify;
pub use prompt::{generate_prompt, PromptOptions};
pub use types::CommandBatch;
pub use types::*;
