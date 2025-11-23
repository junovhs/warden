use crate::config::RuleConfig;
use tree_sitter::{Node, Parser, Query, QueryCursor};

pub struct Analyzer {
    rust_naming: Query,
    rust_safety: Query,
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
            rust_naming: Query::new(tree_sitter_rust::language(), "(function_item name: (identifier) @name)").unwrap(),
            // Updated Safety Query: Now includes boolean logic gates (any, all, is_some, etc.)
            rust_safety: Query::new(tree_sitter_rust::language(), r#"
                (match_expression) @safe
                (if_expression condition: (let_condition)) @safe
                (while_expression condition: (let_condition)) @safe
                (call_expression function: (field_expression field: (field_identifier) @m (#match? @m "^(expect|unwrap_or|unwrap_or_else|unwrap_or_default|ok|err|map_err|any|all|find|is_some|is_none|contains|contains_key)$"))) @safe
                (call_expression function: (identifier) @f (#match? @f "^(Ok|Err)$")) @safe
            "#).unwrap(),

            js_naming: Query::new(tree_sitter_javascript::language(), r"
                (function_declaration name: (identifier) @name)
                (method_definition name: (property_identifier) @name)
                (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])
            ").unwrap(),
            js_safety: Query::new(tree_sitter_javascript::language(), r#"
                (try_statement) @safe
                (call_expression function: (member_expression property: (property_identifier) @m (#eq? @m "catch"))) @safe
            "#).unwrap(),

            py_naming: Query::new(tree_sitter_python::language(), "(function_definition name: (identifier) @name)").unwrap(),
            py_safety: Query::new(tree_sitter_python::language(), "(try_statement) @safe").unwrap(),
        }
    }

    /// Analyzes the content for violations.
    ///
    /// # Panics
    ///
    /// Panics if the Tree-sitter parser fails to initialize the language.
    #[must_use]
    pub fn analyze(&self, lang: &str, content: &str, config: &RuleConfig) -> Vec<Violation> {
        let (language, naming_q, safety_q) = match lang {
            "rs" => (
                tree_sitter_rust::language(),
                &self.rust_naming,
                &self.rust_safety,
            ),
            "js" | "jsx" | "ts" | "tsx" => (
                tree_sitter_typescript::language_typescript(),
                &self.js_naming,
                &self.js_safety,
            ),
            "py" => (
                tree_sitter_python::language(),
                &self.py_naming,
                &self.py_safety,
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
        Self::check_naming(root, content, naming_q, config, &mut violations);
        Self::check_safety(root, content, safety_q, &mut violations);
        violations
    }

    fn check_naming(
        root: Node,
        source: &str,
        query: &Query,
        config: &RuleConfig,
        out: &mut Vec<Violation>,
    ) {
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(query, root, source.as_bytes()) {
            let node = m.captures[0].node;
            let name = node.utf8_text(source.as_bytes()).unwrap_or("?");

            // Count words (snake_case or camelCase)
            let word_count = if name.contains('_') {
                name.split('_').count()
            } else {
                name.chars().filter(|c| c.is_uppercase()).count() + 1
            };

            if word_count > config.max_function_words
                && !config.ignore_naming_on.iter().any(|p| source.contains(p))
            {
                out.push(Violation {
                    row: node.start_position().row,
                    message: format!(
                        "Function '{name}' has {word_count} words (Max: {})",
                        config.max_function_words
                    ),
                    law: "LAW OF BLUNTNESS",
                });
            }
        }
    }

    fn check_safety(root: Node, source: &str, safety_query: &Query, out: &mut Vec<Violation>) {
        // We walk manually to find function boundaries, then query INSIDE them
        let mut cursor = root.walk();
        loop {
            let node = cursor.node();
            let kind = node.kind();

            if (kind.contains("function") || kind.contains("method"))
                && !Self::is_lifecycle(node, source)
            {
                let mut func_cursor = QueryCursor::new();
                // Use matches(...).next().is_none() to check for existence
                if func_cursor
                    .matches(safety_query, node, source.as_bytes())
                    .next()
                    .is_none()
                {
                    // Filter out tiny wrappers (under 5 lines)
                    let rows = node.end_position().row - node.start_position().row;
                    if rows > 5 {
                        out.push(Violation {
                            row: node.start_position().row,
                            message:
                                "Logic block lacks structural safety (try/catch, match, Result)."
                                    .into(),
                            law: "LAW OF PARANOIA",
                        });
                    }
                }
            }

            if !cursor.goto_first_child() {
                while !cursor.goto_next_sibling() {
                    if !cursor.goto_parent() {
                        return;
                    }
                }
            }
        }
    }

    fn is_lifecycle(node: Node, source: &str) -> bool {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
            return matches!(
                name,
                "new" | "default" | "init" | "__init__" | "constructor" | "render" | "main"
            );
        }
        false
    }
}

pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
}
