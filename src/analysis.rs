use crate::checks::{self, Violation};
use crate::config::RuleConfig;
use tree_sitter::{Parser, Query};

pub struct Analyzer {
    rust_naming: Query,
    rust_safety: Query,
    rust_banned: Query,
    js_naming: Query,
    js_safety: Query,
    py_naming: Query,
    py_safety: Query,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    /// Compiles Tree-sitter queries.
    ///
    /// # Panics
    ///
    /// Panics if the internal hardcoded queries are invalid. This implies a development error.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rust_naming: Query::new(tree_sitter_rust::language(), "(function_item name: (identifier) @name)").expect("Invalid Rust naming query"),
            // Safety: Includes match, if let, while let, ?, explicit safety methods, AND Result return types.
            rust_safety: Query::new(tree_sitter_rust::language(), r#"
                (match_expression) @safe
                (if_expression condition: (let_condition)) @safe
                (while_expression condition: (let_condition)) @safe
                (try_expression) @safe
                (call_expression function: (field_expression field: (field_identifier) @m (#match? @m "^(expect|unwrap_or|unwrap_or_else|unwrap_or_default|ok|err|map_err|any|all|find|is_some|is_none|is_ok|is_err)$"))) @safe
                (call_expression function: (identifier) @f (#match? @f "^(Ok|Err)$")) @safe
                (function_item return_type: (_) @ret (#match? @ret "Result")) @safe
            "#).expect("Invalid Rust safety query"),
            // Banned: Explicitly hunt for unwrap() calls
            rust_banned: Query::new(tree_sitter_rust::language(), r#"
                (call_expression function: (field_expression field: (field_identifier) @m (#eq? @m "unwrap"))) @banned
            "#).expect("Invalid Rust banned query"),

            js_naming: Query::new(tree_sitter_javascript::language(), r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ").expect("Invalid JS naming query"),
            js_safety: Query::new(tree_sitter_javascript::language(), r#"
                (try_statement) @safe
                (call_expression function: (member_expression property: (property_identifier) @m (#eq? @m "catch"))) @safe
            "#).expect("Invalid JS safety query"),

            py_naming: Query::new(tree_sitter_python::language(), "(function_definition name: (identifier) @name)").expect("Invalid Python naming query"),
            // Python Safety: Specific checks for 'try', 'not ...', and comparisons against 'None'
            // Uses predicate (#eq? @op "not") instead of literal "not" to avoid QueryError in v0.20
            py_safety: Query::new(tree_sitter_python::language(), r#"
                (try_statement) @safe
                (if_statement condition: (unary_operator (_) @op (#eq? @op "not"))) @safe
                (if_statement condition: (comparison_operator (_) (none))) @safe
            "#).expect("Invalid Python safety query"),
        }
    }

    /// Analyzes the content for violations.
    ///
    /// # Panics
    ///
    /// Panics if the Tree-sitter parser fails to initialize the language.
    #[must_use]
    pub fn analyze(
        &self,
        lang: &str,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let (language, naming_q, safety_q, banned_q) = match lang {
            "rs" => (
                tree_sitter_rust::language(),
                &self.rust_naming,
                &self.rust_safety,
                Some(&self.rust_banned),
            ),
            "js" | "jsx" | "ts" | "tsx" => (
                tree_sitter_typescript::language_typescript(),
                &self.js_naming,
                &self.js_safety,
                None,
            ),
            "py" => (
                tree_sitter_python::language(),
                &self.py_naming,
                &self.py_safety,
                None,
            ),
            _ => return vec![],
        };

        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to load language");
        let tree = parser.parse(content, None).expect("Failed to parse");
        let root = tree.root_node();

        let mut violations = Vec::new();

        // Delegated checks to `checks.rs` module
        let _ = checks::check_naming(root, content, filename, naming_q, config, &mut violations);
        let _ = checks::check_safety(root, content, safety_q, &mut violations);

        if let Some(bq) = banned_q {
            let _ = checks::check_banned(root, content, bq, &mut violations);
        }

        violations
    }
}
