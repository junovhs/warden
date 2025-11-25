// src/checks.rs
use crate::config::RuleConfig;
use crate::metrics;
use crate::types::Violation;
use anyhow::Result;
use tree_sitter::{Node, Query, QueryCursor};

pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
}

/// Checks for naming violations.
pub fn check_naming(ctx: &CheckContext, query: &Query, out: &mut Vec<Violation>) {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(query, ctx.root, ctx.source.as_bytes()) {
        let node = m.captures[0].node;
        let name = node.utf8_text(ctx.source.as_bytes()).unwrap_or("?");

        if is_ignored(ctx.filename, &ctx.config.ignore_naming_on) {
            continue;
        }

        let word_count = count_words(name);
        if word_count > ctx.config.max_function_words {
            out.push(Violation {
                row: node.start_position().row,
                message: format!(
                    "Function '{name}' has {word_count} words (Max: {}). Is it doing too much?",
                    ctx.config.max_function_words
                ),
                law: "LAW OF BLUNTNESS",
            });
        }
    }
}

fn count_words(name: &str) -> usize {
    if name.contains('_') {
        name.split('_').count()
    } else {
        let caps = name.chars().filter(|c| c.is_uppercase()).count();
        if name.chars().next().is_some_and(char::is_uppercase) {
            caps
        } else {
            caps + 1
        }
    }
}

fn is_ignored(filename: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| filename.contains(p))
}

/// Checks for safety violations.
pub fn check_safety(ctx: &CheckContext, _safety_query: &Query, out: &mut Vec<Violation>) {
    let _ = ctx;
    let _ = out;
}

/// Checks for complexity metrics.
pub fn check_metrics(ctx: &CheckContext, complexity_query: &Query, out: &mut Vec<Violation>) {
    traverse_nodes(ctx, |node| {
        if node.kind().contains("function") || node.kind().contains("method") {
            validate_arity(node, ctx.config.max_function_args, out);
            validate_depth(node, ctx.config.max_nesting_depth, out);
            validate_complexity(
                node,
                ctx.source,
                complexity_query,
                ctx.config.max_cyclomatic_complexity,
                out,
            );
        }
    });
}

fn validate_arity(node: Node, max: usize, out: &mut Vec<Violation>) {
    let args = metrics::count_arguments(node);
    if args > max {
        out.push(Violation {
            row: node.start_position().row,
            message: format!(
                "High Arity: Function takes {args} arguments (Max: {max}). Use a Struct."
            ),
            law: "LAW OF COMPLEXITY",
        });
    }
}

fn validate_depth(node: Node, max: usize, out: &mut Vec<Violation>) {
    let depth = metrics::calculate_max_depth(node);
    if depth > max {
        out.push(Violation {
            row: node.start_position().row,
            message: format!("Deep Nesting: Max depth is {depth} (Max: {max}). Extract logic."),
            law: "LAW OF COMPLEXITY",
        });
    }
}

fn validate_complexity(
    node: Node,
    source: &str,
    query: &Query,
    max: usize,
    out: &mut Vec<Violation>,
) {
    let score = metrics::calculate_complexity(node, source, query);
    if score > max {
        out.push(Violation {
            row: node.start_position().row,
            message: format!("High Complexity: Score is {score} (Max: {max}). Hard to test."),
            law: "LAW OF COMPLEXITY",
        });
    }
}

/// Checks for banned constructs.
/// # Errors
/// Returns `Ok` on success. Errors are reserved for future query failures.
#[allow(clippy::unnecessary_wraps)]
pub fn check_banned(
    ctx: &CheckContext,
    banned_query: &Query,
    out: &mut Vec<Violation>,
) -> Result<()> {
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(banned_query, ctx.root, ctx.source.as_bytes()) {
        let node = m.captures[0].node;
        out.push(Violation {
            row: node.start_position().row,
            message: "Explicit 'unwrap()' call detected. Use 'expect', 'unwrap_or', or '?'.".into(),
            law: "LAW OF PARANOIA",
        });
    }
    Ok(())
}

fn traverse_nodes<F>(ctx: &CheckContext, mut cb: F)
where
    F: FnMut(Node),
{
    let mut cursor = ctx.root.walk();
    loop {
        cb(cursor.node());
        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return;
                }
            }
        }
    }
}
