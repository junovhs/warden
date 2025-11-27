// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};

const MARKDOWN_PATTERNS: &[&str] = &[
    "```", // Standard markdown code blocks
    "~~~", // Alternative markdown code blocks
    "```rust",
    "```python",
    "```javascript",
    "```typescript",
    "```html",
    "```css",
    "```json",
    "```yaml",
    "```toml",
    "```bash",
    "```sh",
    "```sql",
    "```xml",
];

#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let missing = check_missing(manifest, extracted);
    let errors = check_content(extracted);

    if !missing.is_empty() || !errors.is_empty() {
        let ai_message = messages::format_ai_rejection(&missing, &errors);
        return ApplyOutcome::ValidationFailure {
            errors,
            missing,
            ai_message,
        };
    }

    let written = extracted.keys().cloned().collect();
    ApplyOutcome::Success {
        written,
        backed_up: true,
    }
}

fn check_missing(manifest: &Manifest, extracted: &ExtractedFiles) -> Vec<String> {
    let mut missing = Vec::new();
    for entry in manifest {
        if entry.operation != Operation::Delete && !extracted.contains_key(&entry.path) {
            missing.push(entry.path.clone());
        }
    }
    missing
}

fn check_content(extracted: &ExtractedFiles) -> Vec<String> {
    let mut errors = Vec::new();
    for (path, file) in extracted {
        check_single_file(path, &file.content, &mut errors);
    }
    errors
}

fn check_single_file(path: &str, content: &str, errors: &mut Vec<String>) {
    if content.trim().is_empty() {
        errors.push(format!("{path} is empty"));
        return;
    }

    if let Some(pattern) = detect_markdown_block(content) {
        errors.push(format!(
            "{path} contains markdown code block '{pattern}' - \
            AI output must use <file> tags, not markdown"
        ));
    }
}

fn detect_markdown_block(content: &str) -> Option<&'static str> {
    MARKDOWN_PATTERNS
        .iter()
        .find(|&&pattern| content.contains(pattern))
        .copied()
}
