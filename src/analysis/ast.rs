// src/analysis/ast.rs
use super::checks::{self, CheckContext};
use crate::config::RuleConfig;
use crate::types::Violation;
use tree_sitter::{Language, Parser, Query};

pub struct Analyzer {
    rust_naming: Query,
    rust_complexity: Query,
    rust_banned: Query,
    js_naming: Query,
    js_complexity: Query,
    py_naming: Query,
    py_complexity: Query,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rust_naming: compile_query(
                tree_sitter_rust::language(),
                "(function_item name: (identifier) @name)",
            ),
            rust_complexity: compile_query(
                tree_sitter_rust::language(),
                r#"
                (if_expression) @branch
                (match_arm) @branch
                (while_expression) @branch
                (for_expression) @branch
                (binary_expression operator: ["&&" "||"]) @branch
            "#,
            ),
            rust_banned: compile_query(
                tree_sitter_rust::language(),
                r"(call_expression function: (field_expression field: (field_identifier) @method)) @call",
            ),
            js_naming: compile_query(
                tree_sitter_typescript::language_typescript(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            ),
            js_complexity: compile_query(
                tree_sitter_typescript::language_typescript(),
                r#"
                (if_statement) @branch
                (for_statement) @branch
                (for_in_statement) @branch
                (while_statement) @branch
                (do_statement) @branch
                (switch_case) @branch
                (catch_clause) @branch
                (ternary_expression) @branch
                (binary_expression operator: ["&&" "||" "??"]) @branch
            "#,
            ),
            py_naming: compile_query(
                tree_sitter_python::language(),
                "(function_definition name: (identifier) @name)",
            ),
            py_complexity: compile_query(
                tree_sitter_python::language(),
                r"
                (if_statement) @branch
                (for_statement) @branch
                (while_statement) @branch
                (except_clause) @branch
                (boolean_operator) @branch
            ",
            ),
        }
    }

    #[must_use]
    pub fn analyze(
        &self,
        lang: &str,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let Some(queries) = self.select_language(lang) else {
            return vec![];
        };
        Self::run_analysis(&queries, filename, content, config)
    }

    fn select_language(&self, lang: &str) -> Option<LanguageQueries<'_>> {
        match lang {
            "rs" => Some(self.queries_rust()),
            "js" | "jsx" | "ts" | "tsx" => Some(self.queries_js()),
            "py" => Some(self.queries_python()),
            _ => None,
        }
    }

    fn queries_rust(&self) -> LanguageQueries<'_> {
        LanguageQueries {
            language: tree_sitter_rust::language(),
            naming: &self.rust_naming,
            complexity: &self.rust_complexity,
            banned: Some(&self.rust_banned),
        }
    }

    fn queries_js(&self) -> LanguageQueries<'_> {
        LanguageQueries {
            language: tree_sitter_typescript::language_typescript(),
            naming: &self.js_naming,
            complexity: &self.js_complexity,
            banned: None,
        }
    }

    fn queries_python(&self) -> LanguageQueries<'_> {
        LanguageQueries {
            language: tree_sitter_python::language(),
            naming: &self.py_naming,
            complexity: &self.py_complexity,
            banned: None,
        }
    }

    fn run_analysis(
        queries: &LanguageQueries<'_>,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let mut parser = Parser::new();
        if parser.set_language(queries.language).is_err() {
            return vec![];
        }

        let Some(tree) = parser.parse(content, None) else {
            return vec![];
        };

        let mut violations = Vec::new();
        let ctx = CheckContext {
            root: tree.root_node(),
            source: content,
            filename,
            config,
        };

        checks::check_naming(&ctx, queries.naming, &mut violations);
        checks::check_metrics(&ctx, queries.complexity, &mut violations);

        if let Some(banned) = queries.banned {
            checks::check_banned(&ctx, banned, &mut violations);
        }

        violations
    }
}

struct LanguageQueries<'a> {
    language: Language,
    naming: &'a Query,
    complexity: &'a Query,
    banned: Option<&'a Query>,
}

fn compile_query(lang: Language, pattern: &str) -> Query {
    match Query::new(lang, pattern) {
        Ok(q) => q,
        Err(e) => panic!("Invalid tree-sitter query pattern: {e}"),
    }
}