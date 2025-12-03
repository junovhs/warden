// src/trace/mod.rs
//! The `warden trace` command - Smart context generation.

mod options;
mod output;
mod runner;

pub use options::TraceOptions;
pub use runner::{map, run, TraceResult};
