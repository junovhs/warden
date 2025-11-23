use crate::error::Result;
use crate::tokens::Tokenizer;
use colored::Colorize;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

// --- CONFIGURATION ---
const TOKEN_LIMIT: usize = 2000;
const WORD_LIMIT: usize = 3;

pub struct RuleEngine {
    rust: Query,
    python: Query,
    typescript: Query,
    javascript: Query,
}

impl RuleEngine {
    /// Creates a new rule engine.
    ///
    /// # Panics
    ///
    /// Panics if the internal Tree-sitter queries are invalid.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rust: Query::new(
                tree_sitter_rust::language(),
                "(function_item name: (identifier) @name)",
            )
            .unwrap(),
            python: Query::new(
                tree_sitter_python::language(),
                "(function_definition name: (identifier) @name)",
            )
            .unwrap(),
            typescript: Query::new(
                tree_sitter_typescript::language_typescript(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            )
            .unwrap(),
            javascript: Query::new(
                tree_sitter_javascript::language(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            )
            .unwrap(),
        }
    }

    /// Checks a file for rule violations.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be read.
    pub fn check_file(&self, path: &Path) -> Result<bool> {
        let Ok(content) = fs::read_to_string(path) else {
            return Ok(true);
        };

        if content.contains("// warden:ignore") || content.contains("# warden:ignore") {
            return Ok(true);
        }

        let mut passed = true;
        let filename = path.to_string_lossy();

        // 1. TOKEN COUNT
        let token_count = Tokenizer::count(&content);
        if token_count > TOKEN_LIMIT {
            println!(
                "{} {}: {} tokens (Limit: {}). Split this file.",
                "[BLOAT]".red().bold(),
                filename,
                token_count,
                TOKEN_LIMIT
            );
            passed = false;
        }

        // 2. AST ANALYSIS
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => Self::analyze_tree(
                    tree_sitter_rust::language(),
                    &self.rust,
                    &content,
                    &filename,
                    "_",
                    &mut passed,
                ),
                "py" => Self::analyze_tree(
                    tree_sitter_python::language(),
                    &self.python,
                    &content,
                    &filename,
                    "_",
                    &mut passed,
                ),
                "ts" | "tsx" => Self::analyze_tree(
                    tree_sitter_typescript::language_typescript(),
                    &self.typescript,
                    &content,
                    &filename,
                    "camel",
                    &mut passed,
                ),
                "js" | "jsx" => Self::analyze_tree(
                    tree_sitter_javascript::language(),
                    &self.javascript,
                    &content,
                    &filename,
                    "camel",
                    &mut passed,
                ),
                _ => {}
            }
        }

        Ok(passed)
    }

    fn analyze_tree(
        language: Language,
        query: &Query,
        content: &str,
        filename: &str,
        naming_style: &str,
        passed: &mut bool,
    ) {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Error loading grammar");

        let tree = parser.parse(content, None).expect("Error parsing file");
        let root = tree.root_node();

        // A. NAMING
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(query, root, content.as_bytes()) {
            for capture in m.captures {
                let name_bytes = &content.as_bytes()[capture.node.byte_range()];
                let name = String::from_utf8_lossy(name_bytes);

                if naming_style == "camel" {
                    let caps = name.chars().filter(|c| c.is_uppercase()).count();
                    // Collapsed if block
                    if caps + 1 > WORD_LIMIT && !name.chars().next().unwrap_or('a').is_uppercase() {
                        Self::report_naming(filename, &name, passed);
                    }
                } else if name.split('_').count() > WORD_LIMIT {
                    Self::report_naming(filename, &name, passed);
                }
            }
        }

        // B. SAFETY (Recursive walk)
        Self::check_safety_recursive(root, content, filename, passed);
    }

    fn report_naming(filename: &str, name: &str, passed: &mut bool) {
        println!(
            "{} {}: Function '{}' is too complex (Limit: 3 words).",
            "[NAMING]".red().bold(),
            filename,
            name
        );
        *passed = false;
    }

    fn check_safety_recursive(node: Node, content: &str, filename: &str, passed: &mut bool) {
        let kind = node.kind();

        let is_func_body =
            kind.contains("block") || kind == "function_definition" || kind == "arrow_function";

        if is_func_body {
            let code_bytes = &content.as_bytes()[node.byte_range()];
            let code_str = String::from_utf8_lossy(code_bytes).to_lowercase();

            // Skip short functions
            if code_str.lines().count() < 5 {
                return;
            }

            let has_safety = code_str.contains("result")
                || code_str.contains("option")
                || code_str.contains("try")
                || code_str.contains("catch")
                || code_str.contains("except")
                || code_str.contains("match")
                || code_str.contains("unwrap_or")
                || code_str.contains("ok(");

            if !has_safety {
                println!(
                    "{} {}: Logic block missing explicit safety (try/catch/Result).",
                    "[UNSAFE]".yellow().bold(),
                    filename
                );
                *passed = false;
            }
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_safety_recursive(child, content, filename, passed);
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
