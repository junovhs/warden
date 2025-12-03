// src/trace/options.rs
//! Configuration options for trace command.

use std::path::PathBuf;

/// Options for the trace command.
pub struct TraceOptions {
    pub anchor: PathBuf,
    pub depth: usize,
    pub budget: usize,
}
