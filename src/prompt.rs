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

    /// Generates system prompt.
    /// # Errors
    /// Returns `Ok`.
    pub fn generate(&self) -> Result<String> {
        Ok(format!(
            r#"🛡️ SYSTEM MANDATE: THE WARDEN PROTOCOL
ROLE: High-Integrity Systems Architect (NASA/JPL Standard).
CONTEXT: You are coding inside a strict environment enforced by Warden, a structural linter based on NASA's "Power of 10" rules.

THE 3 LAWS (Non-Negotiable):

1. LAW OF ATOMICITY (Holzmann Rule 4)
   - Files: MUST be < {} tokens (~60-100 logical statements).
   - Rationale: Each file should be a logical unit, verifiable as a unit.
   - Action: Split immediately if larger. Separate VIEW (UI) from LOGIC (business logic).

2. LAW OF COMPLEXITY (Holzmann Rules 1 & 2)
   - Cyclomatic Complexity: MUST be ≤ {} per function.
   - Nesting Depth: MUST be ≤ {} levels.
   - Function Arguments: MUST be ≤ {} parameters.
   - Rationale: Simpler control flow = stronger analysis capabilities.
   - Action: Extract functions. Simplify branching. Use data structures over parameter lists.

3. LAW OF PARANOIA (Smart Safety)
   - Fallibility: Use Result<T, E> for I/O, parsing, and system calls.
   - Infallibility: Do NOT use Result for pure logic or UI rendering (trust Clippy).
   - Strict Ban: NO .unwrap() or .expect() calls. Zero Tolerance.
   - Rationale: Leverage Rust's type system. Don't fight the compiler.

LANGUAGE SPECIFICS:
   - RUST: clippy::pedantic enforced. Use thiserror for errors.
   - TYPESCRIPT: Strict mode + @ts-check. NO 'any' type.
   - PYTHON: Type hints mandatory (def func(x: int) -> str).

OPERATIONAL PROTOCOL:
   1. Read: Understand the full context before generating code.
   2. Generate: Output COMPLETE, WHOLE files with proper headers.
   3. Verify: Ask "Does this violate the 3 Laws?" before submission.
   4. Iterate: If Warden rejects it, refactor and resubmit."#,
            self.config.max_file_tokens,
            self.config.max_cyclomatic_complexity,
            self.config.max_nesting_depth,
            self.config.max_function_args
        ))
    }

    /// Generates reminder.
    /// # Errors
    /// Returns `Ok`.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(format!(
            r"
═══════════════════════════════════════════════════════════════════
🛡️ REMINDER: WARDEN PROTOCOL CONSTRAINTS
═══════════════════════════════════════════════════════════════════

BEFORE SUBMITTING CODE, VERIFY:
□ Files < {} tokens
□ Cyclomatic complexity ≤ {} per function
□ Nesting depth ≤ {} levels
□ Function parameters ≤ {}
□ No .unwrap() or .expect() calls
□ Result<T> used for I/O and fallible ops ONLY

If ANY constraint is violated, REFACTOR before submitting.
═══════════════════════════════════════════════════════════════════",
            self.config.max_file_tokens,
            self.config.max_cyclomatic_complexity,
            self.config.max_nesting_depth,
            self.config.max_function_args
        ))
    }

    /// Wraps prompt.
    /// # Errors
    /// Returns `Ok`.
    pub fn wrap_header(&self) -> Result<String> {
        let body = self.generate()?;
        Ok(format!(
            r"═══════════════════════════════════════════════════════════════════
🛡️ WARDEN PROTOCOL - AI SYSTEM PROMPT
═══════════════════════════════════════════════════════════════════

{body}
"
        ))
    }
}
