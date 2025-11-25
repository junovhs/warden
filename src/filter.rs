// src/filter.rs
use crate::config::{Config, BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, SECRET_PATTERN};
use crate::error::Result;
use regex::Regex;
use std::path::Path;

pub struct FileFilter {
    config: Config,
    bin_ext_re: Regex,
    secret_re: Regex,
    code_ext_re: Option<Regex>,
    code_bare_re: Option<Regex>,
}

impl FileFilter {
    /// Creates a new filter.
    /// # Errors
    /// Returns error on invalid regex.
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            bin_ext_re: Regex::new(BIN_EXT_PATTERN)?,
            secret_re: Regex::new(SECRET_PATTERN)?,
            code_ext_re: if config.code_only {
                Some(Regex::new(CODE_EXT_PATTERN)?)
            } else {
                None
            },
            code_bare_re: if config.code_only {
                Some(Regex::new(CODE_BARE_PATTERN)?)
            } else {
                None
            },
        })
    }

    #[must_use]
    pub fn filter(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files.into_iter().filter(|p| self.should_keep(p)).collect()
    }

    fn should_keep(&self, path: &Path) -> bool {
        let s = path.to_string_lossy().replace('\\', "/");
        if self.is_ignored(&s) {
            return false;
        }
        if self.config.code_only && !self.is_code(&s) {
            return false;
        }
        self.is_included(&s)
    }

    fn is_ignored(&self, path: &str) -> bool {
        if self.secret_re.is_match(path) {
            return true;
        }
        if self.bin_ext_re.is_match(path) {
            return true;
        }
        if self
            .config
            .exclude_patterns
            .iter()
            .any(|p| p.is_match(path))
        {
            return true;
        }
        false
    }

    fn is_included(&self, path: &str) -> bool {
        self.config.include_patterns.is_empty()
            || self
                .config
                .include_patterns
                .iter()
                .any(|p| p.is_match(path))
    }

    fn is_code(&self, path: &str) -> bool {
        match (&self.code_ext_re, &self.code_bare_re) {
            (Some(ext), Some(bare)) => ext.is_match(path) || bare.is_match(path),
            _ => true,
        }
    }
}
