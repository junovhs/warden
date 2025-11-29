// src/analysis/checks.rs
use super::metrics;
use crate::config::RuleConfig;
use crate::types::Violation;
use tree_sitter::{Node, Query, QueryCursor, QueryMatch, TreeCursor};

pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
}

/// Checks for naming violations (function name word count).
pub fn check_naming(ctx: &CheckContext, query: &Query, out: &mut Vec<Violation>) {
    if is_ignored(ctx.filename, &ctx.config.ignore_naming_on) {
        return;
    }

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(query, ctx.root, ctx.source.as_bytes()) {
        let node = m.captures[0].node;
        let name = node.utf8_text(ctx.source.as_bytes()).unwrap_or("?");
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

/// Checks for complexity metrics (arity, depth, cyclomatic complexity).
pub fn check_metrics(ctx: &CheckContext, complexity_query: &Query, out: &mut Vec<Violation>) {
    traverse_nodes(ctx, |node| {
        let kind = node.kind();
        if kind.contains("function") || kind.contains("method") {
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

/// Checks for banned constructs (`.unwrap()` and `.expect()` calls).
pub fn check_banned(ctx: &CheckContext, banned_query: &Query, out: &mut Vec<Violation>) {
    let mut cursor = QueryCursor::new();
    let names = banned_query.capture_names();

    for m in cursor.matches(banned_query, ctx.root, ctx.source.as_bytes()) {
        process_banned_match(&m, names, ctx, out);
    }
}

fn process_banned_match(
    m: &QueryMatch,
    names: &[String],
    ctx: &CheckContext,
    out: &mut Vec<Violation>,
) {
    let mut method_name: Option<&str> = None;
    let mut row = 0;

    for cap in m.captures {
        let capture_name = &names[cap.index as usize];

        if capture_name == "method" {
            method_name = cap.node.utf8_text(ctx.source.as_bytes()).ok();
        }
        if capture_name == "call" {
            row = cap.node.start_position().row;
        }
    }

    if let Some(name) = method_name {
        if name == "unwrap" || name == "expect" {
            out.push(Violation {
                row,
                message: format!("Banned: '.{name}()'. Use '?' or 'unwrap_or'."),
                law: "LAW OF PARANOIA",
            });
        }
    }
}

fn traverse_nodes<F>(ctx: &CheckContext, mut cb: F)
where
    F: FnMut(Node),
{
    let mut cursor = ctx.root.walk();
    loop {
        cb(cursor.node());
        if !advance_cursor(&mut cursor) {
            break;
        }
    }
}

fn advance_cursor(cursor: &mut TreeCursor) -> bool {
    if cursor.goto_first_child() {
        return true;
    }
    while !cursor.goto_next_sibling() {
        if !cursor.goto_parent() {
            return false;
        }
    }
    true
}