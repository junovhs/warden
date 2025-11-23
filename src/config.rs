use crate::error::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum GitMode {
    Auto,
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub git_mode: GitMode,
    pub include_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<Regex>,
    pub code_only: bool,
    pub verbose: bool,
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            verbose: false,
        }
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`. Reserved for future validation logic
    /// that might return an error if configurations are invalid.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_ignore_file(&mut self) {
        let ignore_path = Path::new(".wardenignore");
        if ignore_path.exists() {
            if let Ok(content) = fs::read_to_string(ignore_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Ok(re) = Regex::new(trimmed) {
                        self.exclude_patterns.push(re);
                    }
                }
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// THE "SMART PRUNE" LIST
pub const PRUNE_DIRS: &[&str] = &[
    // 1. Artifacts & Build Garbage
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "gen",
    "generated",
    ".venv",
    "venv",
    ".tox",
    ".cache",
    "__pycache__",
    "coverage",
    "vendor",
    "third_party",
    // 2. Lockfiles (Noise)
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "Gemfile.lock",
    "composer.lock",
    "poetry.lock",
    // 3. Assets (Binary/Visual Noise)
    "_assets",
    "assets",
    "static",
    "public",
    "media",
    "images",
    "img",
    "fonts",
    "icons",
    "res",
    "resources",
    // 4. Context/Meta (Tests & Docs)
    "tests",
    "test",
    "spec",
    "__tests__",
    "docs",
    "doc",
    "documentation",
    "examples",
    "samples",
];

// Extensions that represent binary data or useless machine-generated text (like .map)
pub const BIN_EXT_PATTERN: &str = r"(?i)\.(png|jpe?g|gif|svg|ico|icns|webp|woff2?|ttf|otf|pdf|mp4|mov|mkv|avi|mp3|wav|flac|zip|gz|bz2|xz|7z|rar|jar|csv|tsv|parquet|sqlite|db|bin|exe|dll|so|dylib|pdb|pkl|onnx|torch|tgz|zst|lock|log|map|min\.js|min\.css)$";

// Credentials detection
pub const SECRET_PATTERN: &str = r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa(\.pub)?|id_ed25519(\.pub)?|.*\.(pem|p12|jks|keystore|pfx))$)";

// Code extension regex (for --code-only mode)
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(c|h|cc|hh|cpp|hpp|rs|go|py|js|jsx|ts|tsx|java|kt|kts|rb|php|scala|cs|swift|m|mm|lua|sh|bash|zsh|fish|ps1|sql|html|xhtml|xml|xsd|xslt|yaml|yml|toml|ini|cfg|conf|json|ndjson|md|rst|tex|s|asm|cmake|gradle|proto|graphql|gql|nix|dart|scss|less|css)$";

pub const CODE_BARE_PATTERN: &str =
    r"(?i)(Makefile|Dockerfile|dockerfile|CMakeLists\.txt|BUILD|WORKSPACE)$";
