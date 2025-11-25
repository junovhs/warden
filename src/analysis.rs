// src/analysis.rs
use crate::checks::{self, CheckContext};
use crate::config::RuleConfig;
use crate::types::Violation;
use anyhow::Result;
use tree_sitter::{Language, Parser, Query};

pub struct Analyzer {
    rust_naming: Query,
    rust_safety: Query,
    rust_complexity: Query,
    rust_banned: Query,
    js_naming: Query,
    js_safety: Query,
    js_complexity: Query,
    py_naming: Query,
    py_safety: Query,
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
            rust_naming: q(
                tree_sitter_rust::language(),
                "(function_item name: (identifier) @name)",
            ),
            rust_safety: q(tree_sitter_rust::language(), r"(match_expression) @safe"),
            rust_complexity: q(
                tree_sitter_rust::language(),
                r#"
                (if_expression) @branch
                (match_arm) @branch
                (while_expression) @branch
                (for_expression) @branch
                (binary_expression operator: ["&&" "||"]) @branch
            "#,
            ),
            rust_banned: q(
                tree_sitter_rust::language(),
                r#"
                (call_expression function: (field_expression field: (field_identifier) @m (#eq? @m "unwrap"))) @banned
            "#,
            ),
            js_naming: q(
                tree_sitter_typescript::language_typescript(),
                r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ",
            ),
            js_safety: q(
                tree_sitter_typescript::language_typescript(),
                r"(try_statement) @safe",
            ),
            js_complexity: q(
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
            py_naming: q(
                tree_sitter_python::language(),
                "(function_definition name: (identifier) @name)",
            ),
            py_safety: q(tree_sitter_python::language(), r"(try_statement) @safe"),
            py_complexity: q(
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
        Self::run_analysis(queries, filename, content, config)
    }

    fn select_language(
        &self,
        lang: &str,
    ) -> Option<(Language, &Query, &Query, &Query, Option<&Query>)> {
        if lang == "rs" {
            return Some(self.queries_rust());
        }
        if matches!(lang, "js" | "jsx" | "ts" | "tsx") {
            return Some(self.queries_js());
        }
        if lang == "py" {
            return Some(self.queries_python());
        }
        None
    }

    fn queries_rust(&self) -> (Language, &Query, &Query, &Query, Option<&Query>) {
        (
            tree_sitter_rust::language(),
            &self.rust_naming,
            &self.rust_safety,
            &self.rust_complexity,
            Some(&self.rust_banned),
        )
    }

    fn queries_js(&self) -> (Language, &Query, &Query, &Query, Option<&Query>) {
        (
            tree_sitter_typescript::language_typescript(),
            &self.js_naming,
            &self.js_safety,
            &self.js_complexity,
            None,
        )
    }

    fn queries_python(&self) -> (Language, &Query, &Query, &Query, Option<&Query>) {
        (
            tree_sitter_python::language(),
            &self.py_naming,
            &self.py_safety,
            &self.py_complexity,
            None,
        )
    }

    fn run_analysis(
        (language, naming, safety, complexity, banned): (
            Language,
            &Query,
            &Query,
            &Query,
            Option<&Query>,
        ),
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let mut parser_instance = Parser::new();
        let Ok(parser) = parser_instance.get_init(language) else {
            return vec![];
        };

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

        checks::check_naming(&ctx, naming, &mut violations);
        checks::check_safety(&ctx, safety, &mut violations);
        checks::check_metrics(&ctx, complexity, &mut violations);

        if let Some(bq) = banned {
            let _ = checks::check_banned(&ctx, bq, &mut violations);
        }

        violations
    }
}

trait ParserInit {
    fn get_init(&mut self, lang: Language) -> Result<&mut Self>;
}

impl ParserInit for Parser {
    fn get_init(&mut self, lang: Language) -> Result<&mut Self> {
        self.set_language(lang)?;
        Ok(self)
    }
}

fn q(lang: Language, pattern: &str) -> Query {
    Query::new(lang, pattern).expect("Invalid Query")
}
