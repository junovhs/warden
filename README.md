# SlopChop

**AI writes slop. You chop it clean.**

---

## The Story

I'm a product designer, not a developer. I built this entire tool by chatting with AI. Every line of Rust came from Claude and ChatGPT.

How? By enforcing rules that keep AI output clean. Small files. Simple functions. No lazy truncation. The AI learns through rejection.

This tool is the proof. It passes its own rules. It was built by the workflow it enables.

---

## What Is This?

SlopChop is the bridge between your AI chat and your codebase.

You love coding with ChatGPT and Claude. The conversation is where the thinking happens. But the last mile sucks:

- Copy code, miss a bracket, broken file
- AI gives you `// rest of implementation`, deletes your code
- 300-line god function you didn't ask for
- Context window forgets everything between sessions

SlopChop fixes all of this.

---

## The Workflow

```
You: "Fix the auth bug"

    slopchop pack src/auth/
    [context copied to clipboard]

You: [paste to Claude] "Here's the code. The login fails when..."

Claude: [responds with code in SlopChop format]

You: [copy response]

    slopchop apply
    
    ✓ 3 files written
    ✓ tests passed  
    ✓ committed
```

If the AI gives you slop:

```
    slopchop apply
    
    ✗ REJECTED
    - src/auth/login.rs: complexity 12 (max 8)
    - src/auth/login.rs: detected "// ..." truncation
    
    [error copied to clipboard]
```

Paste the error back. AI apologizes. Fixes it. Resubmit.

**The AI learns your standards through rejection, not instruction.**

---

## The Killer Feature: Watch Mode

```
slopchop watch
```

Runs in background. Watches your clipboard.

1. You copy from Claude
2. Notification: "3 files ready. ⌘⇧L to land"
3. Press hotkey
4. Done. Never left the browser.

---

## The Three Laws

SlopChop enforces structural constraints. These are what keep AI code from becoming spaghetti.

### Law of Atomicity
Files must be small enough to review.
```
max_file_tokens = 2000  (~500 lines)
```

### Law of Complexity
Functions must be simple enough to test.
```
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
```

### Law of Paranoia (Rust)
No hidden crash paths.
```
.unwrap()  → rejected
.expect()  → rejected
.unwrap_or() → allowed
?          → allowed
```

---

## Installation

```bash
cargo install --path .
```

Then:

```bash
slopchop --init    # interactive setup
```

Or just run `slopchop` and it auto-generates config.

---

## Commands

### Core Workflow

| Command | What it does |
|---------|--------------|
| `slopchop` | Scan codebase for violations |
| `slopchop pack [path]` | Generate context for AI |
| `slopchop apply` | Apply AI response from clipboard |
| `slopchop watch` | Background daemon with hotkey |

### Context Tools

| Command | What it does |
|---------|--------------|
| `slopchop trace <file>` | Pack file + its dependencies |
| `slopchop map` | Show codebase structure |
| `slopchop prompt` | Generate system prompt |

### Project Management

| Command | What it does |
|---------|--------------|
| `slopchop roadmap show` | Display progress |
| `slopchop roadmap apply` | Update roadmap from AI |
| `slopchop roadmap audit` | Verify test coverage |

---

## Configuration

`slopchop.toml`:

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5

[commands]
check = ["cargo test", "cargo clippy -- -D warnings"]
fix = "cargo fmt"
```

---

## The Format

AI outputs code in this format:

```
#__SLOPCHOP_FILE__# src/auth/login.rs
pub fn login(creds: &Credentials) -> Result<Session, AuthError> {
    // complete implementation
    // no truncation
}
#__SLOPCHOP_END__#
```

SlopChop parses this, validates it, writes files atomically, runs tests, commits on success.

If AI uses markdown fences or truncates code, rejected.

---

## Who Is This For?

**Tribe C: AI-Native Builders**

You've fully embraced AI coding. You're not scared of it, you're not skeptical of it. You use it daily. Your pain is the copy-paste friction and the quality inconsistency.

If you think AI code is categorically bad: this isn't for you.

If you think AI code needs guardrails to be good: welcome.

---

## The Proof

This tool was built by a product designer chatting with Claude.

It's ~10,000 lines of Rust across 50+ files. It has tree-sitter parsing, a TUI dashboard, dependency graph analysis, and a roadmap system with test traceability.

Run `slopchop` on this repo. It passes its own rules.

That's the point.

---

## Adoption Tiers

### Tier 1: Quality Scanner
```bash
slopchop          # find violations
slopchop check    # run tests
```
Use it as a linter. No AI required.

### Tier 2: AI Workflow
```bash
slopchop pack     # context for AI
slopchop apply    # land AI code
```
The core loop.

### Tier 3: Full System
```bash
slopchop watch            # daemon mode
slopchop roadmap audit    # test traceability
```
For serious projects.

---

## FAQ

**Is this like Cursor?**

No. Cursor replaces your editor with an AI-integrated IDE. SlopChop doesn't touch your editor. It bridges the gap between any chat UI and your existing workflow. Use it with Claude.ai, ChatGPT, local LLMs, whatever.

**Is this like Copilot?**

No. Copilot is autocomplete. SlopChop is for the conversational workflow where you discuss architecture, debug together, and get back complete files.

**Why Rust?**

Fast, single binary, no runtime dependencies, great tree-sitter support, and the tool enforces the same discipline on itself.

**Can I use this with languages other than Rust?**

Yes. Complexity analysis works for Rust, TypeScript, JavaScript, and Python. Token limits and truncation detection work for any file type.

---

## Chop the Slop

AI generates more code than ever. Most of it is slop.

You can reject AI entirely. You can accept slop and drown in tech debt. Or you can chop it.

```
slopchop watch
```

---

*MIT License*
