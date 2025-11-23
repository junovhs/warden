use crate::config::RuleConfig;
use anyhow::Result;
use tree_sitter::{Node, Query, QueryCursor};

pub struct Violation {
    pub row: usize,
    pub message: String,
    pub law: &'static str,
}

/// Checks for function naming violations based on word count.
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency with the Law of Paranoia, allowing for future error propagation if needed.
// Allow unnecessary wraps to satisfy Warden's Architecture (Law of Paranoia: Must return Result)
#[allow(clippy::unnecessary_wraps)]
pub fn check_naming(
    root: Node,
    source: &str,
    filename: &str,
    query: &Query,
    config: &RuleConfig,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(query, root, source.as_bytes()) {
        let node = m.captures[0].node;
        let name = node.utf8_text(source.as_bytes()).unwrap_or("?");

        let word_count = if name.contains('_') {
            name.split('_').count()
        } else {
            let cap_count = name.chars().filter(|c| c.is_uppercase()).count();
            if name.chars().next().is_some_and(char::is_uppercase) {
                cap_count // PascalCase
            } else {
                cap_count + 1 // camelCase
            }
        };

        let should_ignore = config.ignore_naming_on.iter().any(|p| filename.contains(p));

        if word_count > config.max_function_words && !should_ignore {
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
    Ok(())
}

/// Checks for structural safety (explicit error handling) in logic blocks.
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn check_safety(
    root: Node,
    source: &str,
    safety_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();

        if (kind.contains("function") || kind.contains("method")) && !is_lifecycle(node, source) {
            let mut func_cursor = QueryCursor::new();
            if func_cursor
                .matches(safety_query, node, source.as_bytes())
                .next()
                .is_none()
            {
                let rows = node.end_position().row - node.start_position().row;
                // Only enforce safety on non-trivial functions
                if rows > 5 {
                    out.push(Violation {
                        row: node.start_position().row,
                        message:
                            "Logic block lacks structural safety (try/catch, match, Result, ?)."
                                .into(),
                        law: "LAW OF PARANOIA",
                    });
                }
            }
        }

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return Ok(());
                }
            }
        }
    }
}

/// Checks for banned explicit panic calls like `unwrap()`.
///
/// # Errors
///
/// Currently always returns `Ok`. The `Result` return type is preserved for architectural
/// consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn check_banned(
    root: Node,
    source: &str,
    banned_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(banned_query, root, source.as_bytes()) {
        let node = m.captures[0].node;
        out.push(Violation {
            row: node.start_position().row,
            message: "Explicit 'unwrap()' call detected. Use 'expect', 'unwrap_or', or '?'.".into(),
            law: "LAW OF PARANOIA",
        });
    }
    Ok(())
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
