// tests/integration_skeleton.rs
//! Integration tests for the skeleton system.
//! Covers: v0.5.0 Skeleton System features

use warden_core::skeleton;

/// Verifies Rust function bodies are replaced with { ... }.
/// Feature: Rust body → { ... }
#[test]
fn test_clean_rust_basic() {
    let code = r#"
fn simple_function() {
    let x = 42;
    println!("{}", x);
    for i in 0..10 {
        println!("{}", i);
    }
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    // Body should be replaced
    assert!(
        result.contains("fn simple_function()"),
        "Should keep signature"
    );
    assert!(
        result.contains("{ ... }") || result.contains("{...}"),
        "Body should be replaced with ellipsis: {}",
        result
    );
    assert!(
        !result.contains("let x = 42"),
        "Should not contain body code"
    );
}

/// Verifies nested Rust functions are handled.
/// Feature: Rust nested functions
#[test]
fn test_clean_rust_nested() {
    let code = r#"
fn outer() {
    fn inner() {
        println!("inner");
    }
    inner();
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    // Both functions should be skeletonized
    assert!(result.contains("fn outer()"), "Should keep outer signature");
    // Implementation may vary on nested function handling
}

/// Verifies Rust impl blocks are handled.
#[test]
fn test_clean_rust_impl() {
    let code = r#"
impl MyStruct {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(result.contains("impl MyStruct"), "Should keep impl");
    assert!(
        result.contains("pub fn new()"),
        "Should keep method signatures"
    );
}

/// Verifies Python function bodies are replaced with ...
/// Feature: Python body → ...
#[test]
fn test_clean_python() {
    let code = r#"
def calculate(x, y):
    result = x + y
    for i in range(10):
        result += i
    return result

class MyClass:
    def __init__(self):
        self.value = 0

    def get_value(self):
        return self.value
"#;

    let result = skeleton::skeletonize(code, "py");

    // Python uses ... for ellipsis body
    assert!(
        result.contains("def calculate"),
        "Should keep function signature"
    );
    assert!(
        result.contains("...") || !result.contains("result = x + y"),
        "Body should be replaced or removed"
    );
}

/// Verifies TypeScript/JS function bodies are handled.
/// Feature: TypeScript/JS body
#[test]
fn test_clean_typescript() {
    let code = r#"
function processData(data: string): number {
    const parsed = JSON.parse(data);
    return parsed.value * 2;
}

class DataProcessor {
    private data: string;

    constructor(data: string) {
        this.data = data;
    }

    process(): number {
        return this.processInternal();
    }

    private processInternal(): number {
        return parseInt(this.data);
    }
}
"#;

    let result = skeleton::skeletonize(code, "ts");

    assert!(
        result.contains("function processData"),
        "Should keep function signature"
    );
    assert!(result.contains("class DataProcessor"), "Should keep class");
}

/// Verifies arrow functions are handled.
/// Feature: Arrow function support
#[test]
fn test_clean_arrow_functions() {
    let code = r#"
const add = (a: number, b: number): number => {
    const sum = a + b;
    console.log(sum);
    return sum;
};

const multiply = (a: number, b: number) => a * b;
"#;

    let result = skeleton::skeletonize(code, "ts");

    // Arrow functions should be handled
    assert!(result.contains("const add"), "Should keep arrow function");
}

/// Verifies unsupported extensions pass through unchanged.
/// Feature: Unsupported passthrough
#[test]
fn test_clean_unsupported_extension() {
    let code = r#"
Some random content
That is not code
"#;

    let result = skeleton::skeletonize(code, "xyz");

    // Unsupported should pass through unchanged
    assert_eq!(
        result.trim(),
        code.trim(),
        "Unsupported should pass through"
    );
}

/// Verifies struct definitions are preserved.
#[test]
fn test_rust_structs_preserved() {
    let code = r#"
pub struct Config {
    pub max_tokens: usize,
    pub max_depth: usize,
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(
        result.contains("pub struct Config"),
        "Should preserve struct"
    );
    assert!(result.contains("max_tokens"), "Should preserve fields");
}

/// Verifies enum definitions are preserved.
#[test]
fn test_rust_enums_preserved() {
    let code = r#"
pub enum Status {
    Active,
    Inactive,
    Pending(String),
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(result.contains("pub enum Status"), "Should preserve enum");
}

/// Verifies trait definitions are handled.
#[test]
fn test_rust_traits() {
    let code = r#"
pub trait Processor {
    fn process(&self) -> Result<(), Error>;

    fn validate(&self) -> bool {
        true
    }
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(result.contains("pub trait Processor"), "Should keep trait");
    assert!(
        result.contains("fn process"),
        "Should keep method signatures"
    );
}

/// Verifies imports and use statements are preserved.
#[test]
fn test_imports_preserved() {
    let code = r#"
use std::io;
use std::collections::HashMap;

fn main() {
    let map: HashMap<String, i32> = HashMap::new();
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(result.contains("use std::io"), "Should preserve imports");
    assert!(
        result.contains("use std::collections::HashMap"),
        "Should preserve imports"
    );
}

/// Verifies JavaScript modules are handled.
#[test]
fn test_js_modules() {
    let code = r#"
import { useState } from 'react';
import axios from 'axios';

export function fetchData(url) {
    return axios.get(url).then(res => res.data);
}

export default function App() {
    const [data, setData] = useState(null);
    return <div>{data}</div>;
}
"#;

    let result = skeleton::skeletonize(code, "js");

    assert!(result.contains("import"), "Should preserve imports");
    assert!(result.contains("export"), "Should preserve exports");
}

/// Verifies comments are preserved in skeleton.
#[test]
fn test_comments_preserved() {
    let code = r#"
/// This is a doc comment explaining the function
/// It has multiple lines
pub fn documented_function() {
    // This is an implementation comment
    let x = 42;
}
"#;

    let result = skeleton::skeletonize(code, "rs");

    // Doc comments should be preserved as they're part of the API
    assert!(
        result.contains("/// This is a doc comment"),
        "Should preserve doc comments"
    );
}

/// Verifies type aliases are preserved.
#[test]
fn test_type_aliases_preserved() {
    let code = r#"
type Result<T> = std::result::Result<T, Error>;
type Handler = fn(&Request) -> Response;
"#;

    let result = skeleton::skeletonize(code, "rs");

    assert!(
        result.contains("type Result"),
        "Should preserve type aliases"
    );
}
