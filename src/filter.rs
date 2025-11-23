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
    /// Creates a new file filter.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the regex patterns (binary extensions, secrets, or code patterns) fail to compile.
    pub fn new(config: Config) -> Result<Self> {
        let bin_ext_re = Regex::new(BIN_EXT_PATTERN)?;
        let secret_re = Regex::new(SECRET_PATTERN)?;

        let (code_ext_re, code_bare_re) = if config.code_only {
            (
                Some(Regex::new(CODE_EXT_PATTERN)?),
                Some(Regex::new(CODE_BARE_PATTERN)?),
            )
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            bin_ext_re,
            secret_re,
            code_ext_re,
            code_bare_re,
        })
    }

    #[must_use]
    pub fn filter(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files.into_iter().filter(|p| self.should_keep(p)).collect()
    }

    fn should_keep(&self, path: &Path) -> bool {
        let s = path.to_string_lossy().replace('\\', "/");

        // Structural Safety: Explicit truth table matching
        match (self.is_secret(&s), self.is_binary(&s), self.is_excluded(&s)) {
            (true, _, _) | (_, true, _) | (_, _, true) => return false,
            _ => {}
        }

        match (self.is_included(&s), self.config.code_only) {
            (false, _) => false,
            (_, true) if !self.is_code(&s) => false,
            _ => true,
        }
    }

    fn is_secret(&self, path: &str) -> bool {
        self.secret_re.is_match(path)
    }

    fn is_binary(&self, path: &str) -> bool {
        self.bin_ext_re.is_match(path)
    }

    fn is_excluded(&self, path: &str) -> bool {
        self.config
            .exclude_patterns
            .iter()
            .any(|p| p.is_match(path))
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
        // Explicit structural safety for mixed option/regex logic
        match (&self.code_ext_re, &self.code_bare_re) {
            (Some(ext), Some(bare)) => ext.is_match(path) || bare.is_match(path),
            _ => true,
        }
    }
}
