// src/apply/types.rs
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Update,
    New,
    Delete,
}

#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub path: String,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub content: String,
    pub line_count: usize,
}

#[derive(Debug)]
pub enum ApplyOutcome {
    Success {
        written: Vec<String>,
        backed_up: bool,
    },
    ValidationFailure {
        errors: Vec<String>,
        missing: Vec<String>,
        ai_message: String,
    },
    ParseError(String),
    WriteError(String),
}

/// Configuration options for the apply process.
/// Created to satisfy Law of Complexity (max 5 args).
#[derive(Debug, Clone, Copy, Default)]
pub struct ApplyConfig<'a> {
    pub dry_run: bool,
    pub force: bool,
    pub commit: bool,
    pub root: Option<&'a Path>,
}

// The manifest is just a list of entries
pub type Manifest = Vec<ManifestEntry>;

// The extracted files are mapped by path
pub type ExtractedFiles = HashMap<String, FileContent>;
