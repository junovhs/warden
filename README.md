# Warden

**A code quality gatekeeper for AI-assisted development**

Warden creates a feedback loop between your codebase and AI coding assistants. It packages your code with configurable quality constraints, validates AI responses, and only commits changes that pass your rules.

The pitch: instead of manually reviewing every AI-generated file, Warden automatically rejects malformed output and asks the AI to try again. When everything passes, it commits and pushes. You stay in flow.

---

## What It Actually Does

1. **Pack** — Bundles your codebase into a format AI can consume, with instructions baked in
2. **Apply** — Parses AI responses, validates them against your rules, writes files
3. **Verify** — Runs your test suite and linters automatically
4. **Commit** — If everything passes, commits and pushes. If not, generates feedback for the AI.

The loop looks like this:

```
warden pack → Send to AI → AI responds → warden apply
                                              ↓
                              ✅ Pass → auto-commit & push
                              ❌ Fail → rejection copied to clipboard → paste back to AI
```

---

## The Defaults (And How to Change Them)

Warden ships with sensible defaults for keeping AI-generated code reviewable:

| Rule | Default | What It Catches |
|------|---------|-----------------|
| `max_file_tokens` | 2000 | Files that are too big to reason about |
| `max_cyclomatic_complexity` | 8 | Functions with too many branches to test |
| `max_nesting_depth` | 3 | Deeply nested logic that's hard to follow |
| `max_function_args` | 5 | Functions doing too many things |

**These are just defaults.** Every rule is configurable in `warden.toml`:

```toml
[rules]
max_file_tokens = 3000              # Bump it up for your legacy code
max_cyclomatic_complexity = 12      # More lenient on branching
max_nesting_depth = 4               # Allow deeper nesting
max_function_args = 7               # More args? Sure.

# Skip rules entirely for certain files
ignore_tokens_on = [".md", ".lock", ".py"]
ignore_naming_on = ["tests", "spec"]
```

Want Warden to basically stay out of your way? Crank the limits up. Want it strict for a greenfield project? Dial them down. The engine doesn't care—it just enforces whatever you configure.

### Presets

The `--init` wizard offers three starting points:

| Preset | Tokens | Complexity | Nesting | Use Case |
|--------|--------|------------|---------|----------|
| **Strict** | 1500 | 6 | 2 | New projects, you want tight constraints |
| **Standard** | 2000 | 8 | 3 | Balanced, good for most teams |
| **Relaxed** | 3000 | 12 | 4 | Legacy code, migration projects |

Pick one and adjust from there.

---

## Installation

```bash
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

Verify:
```bash
warden --version
```

---

## Quick Start

### 1. Initialize

```bash
cd your-project
warden --init
```

Or just run `warden`—it'll generate a default config automatically.

### 2. Scan

```bash
warden
```

Reports any violations based on your configured rules.

### 3. Pack for AI

```bash
warden pack
```

Generates `context.txt` with your codebase and copies the file path to clipboard. Attach it to your AI conversation.

### 4. Apply AI Response

Copy the AI's response, then:

```bash
warden apply
```

Warden validates the response, writes files, runs your checks, and commits if everything passes.

---

## The Pack Command

`warden pack` is how you get your codebase into an AI. It outputs everything in "Nabla format"—a simple delimiter system that AIs parse reliably.

```bash
warden pack                    # Generate context.txt, copy path to clipboard
warden pack --copy             # Copy content directly to clipboard
warden pack --stdout           # Print to stdout
warden pack src/main.rs        # Focus on one file, compress the rest
warden pack --skeleton         # Compress everything to signatures only
warden pack --noprompt         # Skip the instruction header
warden pack --git-only         # Only git-tracked files
warden pack --code-only        # Skip docs and config files
```

### Focus Mode

For large codebases, you often want to show the AI one file in full detail while giving it a map of everything else:

```bash
warden pack src/auth/login.rs
```

This includes `login.rs` in full, but "skeletonizes" other files—reducing them to just function signatures. The AI sees the structure without burning context on implementation details it doesn't need.

### Skeleton Mode

```bash
warden pack --skeleton
```

Compresses *all* files to signatures. Useful for "show me the architecture" prompts or when you just need the AI to understand the shape of things.

**Before:**
```rust
pub fn validate_user(input: &UserInput) -> Result<User, ValidationError> {
    let email = input.email.trim();
    if email.is_empty() {
        return Err(ValidationError::EmptyEmail);
    }
    // ... 40 more lines
}
```

**After:**
```rust
pub fn validate_user(input: &UserInput) -> Result<User, ValidationError> { ... }
```

---

## The Apply Command

`warden apply` reads from your clipboard and:

1. **Shows the plan** (if the AI included one) and asks for confirmation
2. **Validates paths** — blocks traversal attacks, absolute paths, sensitive files
3. **Checks for truncation** — rejects `// ...` and "rest of code" placeholders
4. **Backs up existing files** to `.warden_apply_backup/`
5. **Writes changes** to disk
6. **Runs verification** — your configured `check` commands
7. **Commits and pushes** if everything passes

If validation fails, Warden generates a rejection message explaining what went wrong and copies it to your clipboard. Paste it back to the AI.

---

## The Nabla Protocol

AI responses need to be in Nabla format for Warden to parse them. The system prompt (included when you `pack`) teaches this to the AI.

**Why not markdown code fences?**

AIs frequently mess up nested fences or escape them incorrectly. The `∇∇∇` and `∆∆∆` symbols are unambiguous and never appear in normal code.

### Format

```
∇∇∇ PLAN ∇∇∇
GOAL: What you're doing
CHANGES:
1. First change
2. Second change
∆∆∆

∇∇∇ MANIFEST ∇∇∇
src/file1.rs
src/file2.rs [NEW]
src/old.rs [DELETE]
∆∆∆

∇∇∇ src/file1.rs ∇∇∇
// Complete file content
// No truncation allowed
∆∆∆
```

**Rules:**
- Every file in MANIFEST needs a matching file block (unless `[DELETE]`)
- Files must be complete—no `// ...` or `/* remaining */`
- Paths must match exactly

---

## Configuration

### warden.toml

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
max_function_words = 5

# Skip token counting for these patterns
ignore_tokens_on = [".lock", ".md"]

# Skip naming rules for test files
ignore_naming_on = ["tests", "spec"]

[commands]
# Run these during `warden check` and after successful apply
check = [
    "cargo clippy --all-targets -- -D warnings",
    "cargo test"
]

# Run these during `warden fix`
fix = "cargo fmt"
```

### .wardenignore

Exclude files from scanning and packing:

```
target
node_modules
.git
dist/
*.generated.ts
```

### Per-File Ignores

Skip Warden analysis for a specific file:

```rust
// warden:ignore
```
```python
# warden:ignore
```
```html
<!-- warden:ignore -->
```

---

## Rust-Specific: The Paranoia Rule

For Rust projects, Warden also flags `.unwrap()` and `.expect()` calls by default. The idea: these are fine for prototyping but risky in production code.

This is just another configurable behavior. If you're fine with unwraps, you can ignore the warnings or adjust your workflow.

---

## Commands Reference

| Command | What It Does |
|---------|--------------|
| `warden` | Scan and report violations |
| `warden --ui` | Interactive TUI dashboard |
| `warden --init` | Configuration wizard |
| `warden pack` | Generate context for AI |
| `warden apply` | Apply AI response from clipboard |
| `warden check` | Run configured check commands |
| `warden fix` | Run configured fix commands |
| `warden prompt` | Output just the system prompt |

### Roadmap Commands

Warden includes a system for tracking features with test verification:

| Command | What It Does |
|---------|--------------|
| `warden roadmap init` | Create ROADMAP.md template |
| `warden roadmap show` | Display as tree |
| `warden roadmap tasks` | List tasks |
| `warden roadmap audit` | Verify test anchors |

---

## Security

Warden blocks potentially dangerous paths in AI responses:

- **Traversal:** `../` sequences
- **Absolute paths:** `/etc/passwd`, `C:\Windows`
- **Sensitive files:** `.git/`, `.env`, `.ssh/`, `.aws/`, credentials
- **Hidden files:** Dotfiles (except `.gitignore`, `.wardenignore`)

It also detects truncation markers that indicate the AI got lazy:
- `// ...`
- `/* ... */`
- `# ...`
- Phrases like "remaining code" or "rest of implementation"

---

## Language Support

| Language | Complexity Analysis | Skeleton | Notes |
|----------|:------------------:|:--------:|-------|
| Rust | ✅ | ✅ | + `.unwrap()`/`.expect()` detection |
| TypeScript | ✅ | ✅ | |
| JavaScript | ✅ | ✅ | |
| Python | ✅ | ✅ | |
| Go | — | — | Project detection only |
| Other | — | — | Token counting works for anything |

---

## Backups

Before writing any changes, Warden backs up modified files:

```
.warden_apply_backup/
└── 1699876543/        # Timestamp
    └── src/
        └── modified.rs
```

Add `.warden_apply_backup` to your `.gitignore`.

---

## Philosophy

Warden's constraints aren't about code style—they're about making AI output predictable and reviewable.

When an AI generates a 5000-token file with complexity score 15, you can't meaningfully review it. When it's forced to produce smaller, simpler chunks, you can actually see what changed.

The rules are guardrails, not gospel. Tune them to your project's needs.

---

## License

MIT — See [LICENSE](LICENSE)
