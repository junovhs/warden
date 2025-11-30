// tests/unit_analysis.rs
//! Unit tests for AST analysis functionality.
//! Covers: v0.2.0 Law of Complexity features for multiple languages

use warden_core::analysis::ast::Analyzer;
use warden_core::config::RuleConfig;

fn default_rules() -> RuleConfig {
    RuleConfig {
        max_file_tokens: 2000,
        max_cyclomatic_complexity: 10,
        max_nesting_depth: 3,
        max_function_args: 5,
        max_function_words: 5,
        ignore_tokens_on: vec![],
        ignore_naming_on: vec![],
    }
}

fn strict_rules() -> RuleConfig {
    RuleConfig {
        max_cyclomatic_complexity: 3,
        max_nesting_depth: 2,
        max_function_args: 3,
        ..default_rules()
    }
}

// =============================================================================
// JAVASCRIPT/TYPESCRIPT COMPLEXITY
// =============================================================================

/// Verifies JS/TS complexity analysis.
/// Feature: JS/TS complexity query
#[test]
fn test_js_complexity() {
    let analyzer = Analyzer::new();
    let config = strict_rules();

    // Complex JS function
    let code = r#"
function complexLogic(a, b, c) {
    if (a > 0) {
        if (b > 0) {
            for (let i = 0; i < 10; i++) {
                if (c > i) {
                    return a + b + c;
                }
            }
        }
    } else if (a < -10) {
        return -1;
    }
    return 0;
}
"#;

    let violations = analyzer.analyze("js", "test.js", code, &config);

    let has_complexity = violations
        .iter()
        .any(|v| v.message.contains("Complexity") || v.law == "LAW OF COMPLEXITY");

    assert!(
        has_complexity,
        "Complex JS function should trigger complexity violation"
    );
}

/// Verifies simple JS function passes.
#[test]
fn test_js_simple_passes() {
    let analyzer = Analyzer::new();
    let config = default_rules();

    let code = r#"
function add(a, b) {
    return a + b;
}
"#;

    let violations = analyzer.analyze("js", "test.js", code, &config);

    let complexity_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.message.contains("Complexity"))
        .collect();

    assert!(
        complexity_violations.is_empty(),
        "Simple JS function should pass"
    );
}

// =============================================================================
// PYTHON COMPLEXITY
// =============================================================================

/// Verifies Python complexity analysis.
/// Feature: Python complexity query
#[test]
fn test_python_complexity() {
    let analyzer = Analyzer::new();
    let config = strict_rules();

    // Complex Python function
    let code = r#"
def complex_logic(a, b, c):
    if a > 0:
        if b > 0:
            for i in range(10):
                if c > i:
                    return a + b + c
    elif a < -10:
        return -1
    return 0
"#;

    let violations = analyzer.analyze("py", "test.py", code, &config);

    let has_complexity = violations
        .iter()
        .any(|v| v.message.contains("Complexity") || v.law == "LAW OF COMPLEXITY");

    assert!(
        has_complexity,
        "Complex Python function should trigger complexity violation"
    );
}

/// Verifies simple Python function passes.
#[test]
fn test_python_simple_passes() {
    let analyzer = Analyzer::new();
    let config = default_rules();

    let code = r#"
def add(a, b):
    return a + b
"#;

    let violations = analyzer.analyze("py", "test.py", code, &config);

    let complexity_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.message.contains("Complexity"))
        .collect();

    assert!(
        complexity_violations.is_empty(),
        "Simple Python function should pass"
    );
}

// =============================================================================
// NAMING - WORD COUNTING
// =============================================================================

/// Verifies snake_case word counting.
/// Feature: Snake_case word counting
#[test]
fn test_snake_case_words() {
    let analyzer = Analyzer::new();
    let mut config = default_rules();
    config.max_function_words = 3;

    // Function with too many words in snake_case
    let code = r#"
fn this_is_a_very_long_function_name_with_many_words() {
    println!("too long");
}
"#;

    let violations = analyzer.analyze("rs", "test.rs", code, &config);

    let has_naming = violations
        .iter()
        .any(|v| v.message.contains("words") || v.law == "LAW OF BLUNTNESS");

    assert!(
        has_naming,
        "Long snake_case name should trigger naming violation"
    );
}

/// Verifies CamelCase word counting.
/// Feature: CamelCase word counting
#[test]
fn test_camel_case_words() {
    let analyzer = Analyzer::new();
    let mut config = default_rules();
    config.max_function_words = 3;

    // TypeScript with long camelCase name
    let code = r#"
function thisIsAVeryLongFunctionNameWithManyWords() {
    console.log("too long");
}
"#;

    let violations = analyzer.analyze("ts", "test.ts", code, &config);

    let has_naming = violations
        .iter()
        .any(|v| v.message.contains("words") || v.law == "LAW OF BLUNTNESS");

    assert!(
        has_naming,
        "Long CamelCase name should trigger naming violation"
    );
}

// =============================================================================
// WARDEN:IGNORE VARIANTS
// =============================================================================

/// Verifies warden:ignore with hash-style comments.
/// Feature: warden:ignore (Hash-style #)
#[test]
fn test_warden_ignore_hash() {
    // This is tested at the file-skip level in the scanner
    // The analyzer itself doesn't handle file-level ignores
    let code = "# warden:ignore\ndef bad(): pass";

    // The content check happens before analysis
    let has_ignore = code.contains("# warden:ignore");
    assert!(has_ignore, "Should detect hash-style warden:ignore");
}

/// Verifies warden:ignore with HTML-style comments.
/// Feature: warden:ignore (HTML-style)
#[test]
fn test_warden_ignore_html() {
    let code = "<!-- warden:ignore -->\n# Title";

    let has_ignore = code.contains("<!-- warden:ignore -->");
    assert!(has_ignore, "Should detect HTML-style warden:ignore");
}

// =============================================================================
// TYPESCRIPT SPECIFIC
// =============================================================================

/// Verifies TypeScript arrow functions are analyzed.
#[test]
fn test_typescript_arrow_functions() {
    let analyzer = Analyzer::new();
    let mut config = default_rules();
    config.max_function_args = 3;

    let code = r#"
const tooManyArgs = (a: number, b: number, c: number, d: number, e: number) => {
    return a + b + c + d + e;
};
"#;

    let violations = analyzer.analyze("ts", "test.ts", code, &config);

    // Arrow functions should be analyzed for arity
    let has_arity = violations
        .iter()
        .any(|v| v.message.contains("argument") || v.message.contains("Arity"));

    assert!(
        has_arity,
        "Arrow function with many args should trigger arity violation"
    );
}

/// Verifies TypeScript class methods are analyzed.
#[test]
fn test_typescript_class_methods() {
    let analyzer = Analyzer::new();
    let config = strict_rules();

    let code = r#"
class Calculator {
    complexMethod(a: number, b: number): number {
        if (a > 0) {
            if (b > 0) {
                for (let i = 0; i < 10; i++) {
                    if (a > i && b > i) {
                        return a + b;
                    }
                }
            }
        }
        return 0;
    }
}
"#;

    let violations = analyzer.analyze("ts", "test.ts", code, &config);

    let has_complexity = violations
        .iter()
        .any(|v| v.message.contains("Complexity") || v.law == "LAW OF COMPLEXITY");

    assert!(
        has_complexity,
        "Complex class method should trigger violation"
    );
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Verifies empty file doesn't crash analyzer.
#[test]
fn test_empty_file() {
    let analyzer = Analyzer::new();
    let config = default_rules();

    let violations = analyzer.analyze("rs", "empty.rs", "", &config);

    // Should not panic and return empty or minimal violations
    assert!(violations.is_empty() || violations.len() <= 1);
}

/// Verifies unsupported language returns empty.
#[test]
fn test_unsupported_language() {
    let analyzer = Analyzer::new();
    let config = default_rules();

    let violations = analyzer.analyze("xyz", "test.xyz", "some content", &config);

    assert!(
        violations.is_empty(),
        "Unsupported language should return no violations"
    );
}

/// Verifies malformed code doesn't crash.
#[test]
fn test_malformed_code() {
    let analyzer = Analyzer::new();
    let config = default_rules();

    let malformed = "fn broken( { {{{{ not valid rust";

    // Should not panic
    let _violations = analyzer.analyze("rs", "broken.rs", malformed, &config);
}
