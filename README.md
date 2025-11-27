# 🛡️ Warden Protocol

**Structural governance for AI-assisted development.**

> *"The rules are like the seat belts in a car: Initially, using them is perhaps a little uncomfortable, but after a while, it becomes second nature, and not using them is unimaginable."*  
> — Gerard J. Holzmann, NASA/JPL

Warden enforces **NASA "Power of 10" Rules** adapted for modern development. It's not a style linter—it's an architectural MRI that keeps code AI-readable and human-verifiable.

**Languages:** Rust, TypeScript, JavaScript, Python

**Status:** Self-hosting. Warden enforces its own rules on its own codebase.

---

## The Problem

AI-generated code drifts. Functions bloat. Complexity creeps. Context windows overflow.

You paste 50KB into Claude, it generates a 400-line function with 6 levels of nesting, and now you're debugging something neither you nor the AI can reason about.

Warden stops this at the source.

---

## The 3 Laws

### 1. Law of Atomicity
Files must be **< 2000 tokens**.

Small files fit in context windows. Small files are verifiable. When a file grows too large, split it.

### 2. Law of Complexity  
- **Cyclomatic Complexity:** ≤ 5 per function (default, configurable up to 10)
- **Nesting Depth:** ≤ 2 levels (default, configurable up to 4)
- **Function Arguments:** ≤ 5 parameters (use structs)

If you can't read a function in one breath, it's too complex.

### 3. Law of Paranoia
- No `.unwrap()` or `.expect()` (Rust)
- Fallible operations must return `Result`

The type system is your ally. Don't lie to the compiler.

---

## Installation

    cargo install --path .

This installs two binaries: `warden` and `knit`.

---

## Quick Start

    cd your-project
    warden              # Scan for violations (auto-creates warden.toml)
    knit --prompt       # Generate context.txt for AI

That's it. No setup required—Warden detects your project type and configures itself.

---

## The Workflow

Warden isn't just a linter—it's a closed-loop system for AI development.

### 1. Generate Context

    knit --prompt

Creates `context.txt` containing:
- Your codebase (filtered, deduplicated)
- The Warden Protocol system prompt
- Output format specification for AI responses
- Token count

### 2. Chat with AI

Drag `context.txt` into Claude/GPT/Gemini. Ask for changes.

The AI will respond with the **Nabla Protocol**:

    <delivery>
    src/lib.rs
    src/new_module.rs [NEW]
    </delivery>
    
    ∇∇∇ src/lib.rs ∇∇∇
    // complete file contents
    ∆∆∆

### 3. Apply Changes

    warden apply

Warden parses the clipboard (or specifically formatted AI output), validates the changes against the 3 Laws, and writes them to disk.

---

## Configuration (`warden.toml`)

    [rules]
    max_file_tokens = 2000
    max_cyclomatic_complexity = 5
    max_nesting_depth = 2
    max_function_args = 5
    
    [commands]
    check = "cargo clippy"  # or "npm run lint", "ruff check ."
    fix = "cargo fmt"       # or "npm run format"

---

## License

MIT