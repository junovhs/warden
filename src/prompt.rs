// src/prompt.rs
use crate::config::RuleConfig;
use anyhow::Result;

pub struct PromptGenerator {
    config: RuleConfig,
}

impl PromptGenerator {
    #[must_use]
    pub fn new(config: RuleConfig) -> Self {
        Self { config }
    }

    /// Generates the full system prompt.
    ///
    /// # Errors
    /// This function returns `Ok` always, but returns `Result` for API consistency.
    pub fn generate(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Wraps the header for context packs.
    ///
    /// # Errors
    /// This function returns `Ok` always, but returns `Result` for API consistency.
    pub fn wrap_header(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Generates a short reminder prompt.
    ///
    /// # Errors
    /// This function returns `Ok` always, but returns `Result` for API consistency.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(self.build_reminder())
    }

    fn build_system_prompt(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let output_format = build_output_format();

        format!(
            r"🛡️ SYSTEM MANDATE: THE WARDEN PROTOCOL
ROLE: High-Integrity Systems Architect (NASA/JPL Standard).
CONTEXT: You are coding inside a strict environment enforced by Warden.

THE 3 LAWS (Non-Negotiable):

1. LAW OF ATOMICITY
   - Files: MUST be < {tokens} tokens.
   - Action: Split immediately if larger.

2. LAW OF COMPLEXITY
   - Cyclomatic Complexity: MUST be ≤ {complexity} per function.
   - Nesting Depth: MUST be ≤ {depth} levels.
   - Function Arguments: MUST be ≤ {args} parameters.

3. LAW OF PARANOIA
   - Use Result<T, E> for I/O and fallible operations.
   - NO .unwrap() or .expect() calls.

{output_format}
"
        )
    }

    fn build_reminder(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;

        format!(
            r"WARDEN CONSTRAINTS:
□ Files < {tokens} tokens
□ Complexity ≤ {complexity}
□ Nesting ≤ {depth}
□ Args ≤ {args}
□ No .unwrap()
□ Use Nabla Format (∇∇∇ ... ∆∆∆)"
        )
    }
}

fn build_output_format() -> String {
    // We construct the delimiters dynamically to avoid confusing the Warden parser.
    let nabla = "∇";
    let delta = "∆";
    let open = format!("{nabla}{nabla}{nabla}");
    let close = format!("{delta}{delta}{delta}");

    format!(r#"OUTPUT FORMAT (MANDATORY):

1. Explain the changes (Technical Plan) using NABLA PROTOCOL:
   - Must start with "GOAL:"
   - Must include "CHANGES:" list

{open} PLAN {open}
GOAL: Refactor authentication module.
CHANGES:
1. Extract user validation to new file.
2. Update config parser.
{close}

2. Declare the plan (Manifest) using NABLA PROTOCOL:

{open} MANIFEST {open}
path/to/file1.rs
path/to/file2.rs [NEW]
{close}

3. Provide EACH file using NABLA PROTOCOL:

{open} path/to/file1.rs {open}
[file content]
{close}

RULES:
- Do NOT use markdown code blocks (e.g. triple backticks) to wrap the file. The {open} delimiters ARE the fence.
- You MAY use markdown inside the file content.
- Every file in the manifest MUST have a matching {open} block.
- Paths must match exactly.
- Do NOT truncate files (No "// ...").
"#)
}