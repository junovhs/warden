// src/roadmap/mod.rs
#![allow(clippy::pedantic)]
#![allow(clippy::all)]

// Legacy Roadmap v1 Module
//
// Kept primarily for `roadmap_v2/cli/migrate.rs` to parse and convert
// legacy roadmap files. Interactive CLI components have been removed.

pub mod audit;
pub mod cmd_handlers;
pub mod cmd_helpers;
pub mod cmd_parser;
pub mod cmd_runner;
pub mod diff;
pub mod display;
pub mod parser;
pub mod str_utils;
pub mod types;

// Re-export types for backward compatibility during migration
pub use types::{Command, Roadmap, TaskStatus};
pub use str_utils::slugify;