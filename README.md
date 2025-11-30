# 🛡️ Warden Protocol

**Structural governance for AI-assisted development.**

> *"The rules are like the seat belts in a car: Initially, using them is perhaps a little uncomfortable, but after a while, it becomes second nature, and not using them is unimaginable."*
> — Gerard J. Holzmann, NASA/JPL

Warden enforces **NASA "Power of 10" Rules** adapted for the AI coding era. It's not a style linter—it's a containment system that keeps AI-generated code verifiable, modular, and sane.

    cargo install --path .

---

## The Problem

You paste 50KB into Claude. It generates a 400-line function with 6 levels of nesting. Now you're debugging something neither you nor the AI can reason about.

AI-generated code **drifts**:
- Functions bloat
- Complexity creeps
- Context windows overflow
- Hallucinations compound

Warden stops this at the source.

---

## The 3 Laws

### 1. Law of Atomicity
Files must be **< 2000 tokens**.

Small files fit in context windows. Small files are verifiable. Small files can be taken to an AI in isolation and brought back—they just slot in.

### 2. Law of Complexity
- **Cyclomatic Complexity:** ≤ 10 per function
- **Nesting Depth:** ≤ 3 levels
- **Function Arguments:** ≤ 5 parameters

These aren't style preferences. They're **containment protocols**. Low complexity bounds the hallucination surface. Shallow nesting prevents AI losing track of scope.

### 3. Law of Paranoia
- No `.unwrap()` or `.expect()` (Rust)
- Fallible operations must return `Result`

The type system is your ally. Don't let AI lie to the compiler.

---

## Quick Start

    cd your-project
    warden              # Scan for violations (auto-creates warden.toml)
    warden config       # Open the interactive configuration deck
    warden pack --prompt # Generate context.txt for AI

That's it. Warden detects your project type (Rust/Node/Python/Go) and configures itself.

---

## The Workflow

Warden is a closed-loop system for AI development.

### 1. Generate Context

    warden pack --prompt

Creates `context.txt` containing:
- Your codebase (filtered, deduplicated)
- The Warden Protocol system prompt
- Current violations (AI sees what to fix)
- Token count

If **Auto-Copy** is enabled, it's already in your clipboard.

### 2. Chat with AI

Paste the file into Claude/GPT/Gemini. Ask for changes.

The AI responds with structured output:

    ∇∇∇ src/lib.rs ∇∇∇
    // complete file contents
    ∆∆∆
    
    ∇∇∇ src/new_module.rs ∇∇∇
    // complete file contents
    ∆∆∆

### 3. Apply Changes

    warden apply

This:
- Extracts file blocks from clipboard
- **Validates paths** (blocks traversal, sensitive files, hidden files)
- **Rejects truncated output** (unbalanced braces, `// ...` markers)
- Creates timestamped backup
- Writes files atomically
- On failure: copies AI-friendly error to clipboard

### 4. Verify

    warden          # Structural analysis
    warden check    # Run your linter (clippy, biome, ruff)

If violations exist, exit code is non-zero. CI-friendly.

### 5. Iterate

If `warden apply` fails, the error is already in your clipboard. Paste it back to AI, get corrected output, repeat.

---

## Visual Interfaces

### 1. Configuration Flight Deck

    warden config

An interactive TUI for managing your project's strictness and workflow.

**Features:**
*   **Global Protocol:** One-click presets (Strict / Standard / Relaxed).
*   **Threat Analytics:** A live gauge showing your containment integrity score.
*   **Workflow Automation:** Toggle auto-copy, auto-format, and auto-commit.
*   **Visual Themes:** Switch between NASA (High Contrast), Cyberpunk (Neon), and Corporate (Subtle).

### 2. Scan Dashboard

    warden --ui

A visual explorer for scan results. Filter by error, sort by size, and inspect file health.

---

## Configuration

Warden uses `warden.toml`. You can edit it manually or via `warden config`.

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
max_function_words = 5
ignore_naming_on = ["tests", "spec"]
ignore_tokens_on = ["lock", ".md"]

[preferences]
theme = "Cyberpunk"
auto_copy = true
auto_format = false
auto_commit = false
commit_prefix = "AI: "
progress_bars = true

[commands]
check = ["cargo clippy", "cargo test"]
fix = "cargo fmt"
```

### Protocol Levels

| Preset | Tokens | Complexity | Depth | Use Case |
|--------|--------|------------|-------|----------|
| **Strict** | 1500 | 4 | 2 | Mission-critical / Greenfield |
| **Standard** | 2000 | 8 | 3 | Recommended balance |
| **Relaxed** | 3000 | 12 | 4 | Legacy / Prototyping |

---

## Safety Features

### Path Safety Validation

Warden blocks dangerous paths before they touch disk:

| Threat | Example | Status |
|--------|---------|--------|
| Directory traversal | `../etc/passwd` | Blocked |
| Absolute paths | `/etc/passwd`, `C:\Windows` | Blocked |
| Git internals | `.git/config` | Blocked |
| Secrets | `.env`, `.ssh/`, `.aws/` | Blocked |
| Hidden files | `.secrets`, `.private` | Blocked |

### Truncation Detection

AI output often gets cut off. Warden catches:
- Unbalanced `{}`, `[]`, `()`
- Truncation markers: `// ...`, `// rest of file`
- Files ending mid-statement

### Atomic Backups

Before any write: `.warden_apply_backup/TIMESTAMP/`

Your original files are always preserved.

---

## Roadmap Management

Warden includes AI-friendly roadmap management. Instead of AI rewriting your entire roadmap, it sends surgical commands.

1. Run `warden roadmap prompt` (Copies current state + instructions)
2. AI responds with commands:
```
===ROADMAP===
CHECK truncation-detection
ADD v0.5.0 "New feature" AFTER truncation-detection
NOTE auth-system "Needs refactor"
===END===
```
3. Run `warden roadmap apply` (Parses and updates `ROADMAP.md`)

---

## Philosophy

Warden exists because AI-assisted development needs **constraints**, not suggestions.

The original Power of 10 rules were for life-critical systems where bugs kill people. We're not building flight software—but we are building systems that must remain comprehensible through hundreds of AI-human iterations.

**Principles:**

1. **Reject bad input, don't fix it** — Warden is a gatekeeper, not a fixer.

2. **Git is the undo system** — Don't reinvent version control.

3. **Explicit > Magic** — If AI doesn't follow format, fail loudly.

4. **Containment over craftsmanship** — For AI, constraints aren't style. They're safety.

5. **Eat your own dogfood** — Warden enforces its own rules on its own codebase.

---

## Self-Hosting

Warden is self-hosting. Run `warden` in this repo:

    ✅ All Clear. Scanned 37586 tokens in 116ms.

The tool passes its own rules.

---

## License

MIT

---

## Links

- [Roadmap](ROADMAP.md)
- [NASA Power of 10 Rules](https://en.wikipedia.org/wiki/The_Power_of_10:_Rules_for_Developing_Safety-Critical_Code)

---

*Complexity is the enemy. Warden is the checkpoint.*