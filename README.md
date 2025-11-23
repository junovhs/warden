# 🛡️ Warden Protocol

**Architecture Governance for the AI Era.**

> *"We do not ask the AI to write good code. We enforce good code via mechanical constraints."*

Warden is a local toolchain designed to enforce **Code With Intent (POT)**. It solves the "Context Drift" and "Hallucination" problems common in AI coding by enforcing strict structural discipline (Atomicity, Naming, Safety) before code is committed.

**v0.3.0 Update:** Warden now uses **Tree-sitter** for structural AST analysis and **Tiktoken** for LLM-native limits. Logic checks are now semantic, not just text-based.

## The Ecosystem

This repository contains two binaries that share a single logic core:

1.  **`warden` (The Enforcer):** An AST-based linter that rejects bloat (tokens), complexity (naming), and unsafe code (scope analysis).
2.  **`knit` (The Messenger):** A smart context-packer that serializes your repository for AI consumption, reporting exactly how many tokens you are feeding the model.

---

## 1. The Warden (Linter)

Warden checks if your code is **maintainable**. It enforces the "3 Laws" of this architecture:

### The 3 Laws
1.  **The Law of Atomicity (Anti-Bloat)**
    *   **Rule:** No file may exceed **2000 Tokens** (approx. 200-250 lines of dense code).
    *   **Goal:** Forces modularity based on **Attention Span**.
2.  **The Law of Bluntness (Naming)**
    *   **Rule:** Function names must be **≤ 3 words** (e.g., `fetchUser` ✅, `fetchUserAndSaveToDb` ❌).
    *   **Goal:** Enforces Single Responsibility Principle (SRP).
3.  **The Law of Paranoia (Safety)**
    *   **Rule:** Logic bodies must contain explicit error handling (`Result`, `try/catch`, `match`, `unwrap_or`).
    *   **Goal:** Prevents "Silent Failures." Warden uses **Tree-sitter** to verify that safety exists structurally within the AST.

### Usage
```bash
# Run inside any Git repo
warden
```

**Bypass:** To intentionally skip a file, add `// warden:ignore` to the top.

---

## 2. Knit (Context Packer)

Knit stitches your "Atomic" files into a single stream for LLMs.

### Usage
```bash
# Standard Text Output
knit

# XML Output (Optimized for Claude 3.5 / Newer Models)
knit --format xml
```

---

## ⚙️ Configuration

Warden works out-of-the-box, but you can customize the "3 Laws" via `warden.toml` in your project root.

```toml
# warden.toml
[rules]
max_file_tokens = 2500      # Default: 2000
max_function_words = 4      # Default: 3
ignore_naming_on = ["tests"] # paths containing this string skip naming checks
```

To ignore specific files/folders, create a `.wardenignore` file:
```text
# .wardenignore
legacy_code/
experiments/
```

---

## 🤖 The AI System Prompt

To make the AI obey Warden, paste this into your System Prompt / Custom Instructions:

```text
ROLE: High-Integrity Systems Architect.
CONTEXT: You are coding inside a strict "Code With Intent" environment enforced by a binary linter called Warden.

THE 3 LAWS (Non-Negotiable):
1. LAW OF ATOMICITY (Token Limits):
   - Files MUST be < 2000 Tokens (~200 lines).
   - If a file grows too large, split it immediately.

2. LAW OF PARANOIA (Scope Safety):
   - Logic Blocks MUST contain explicit error handling (Result, try/catch, Option) INSIDE the function body.
   - No unwrap() allowed.

3. LAW OF BLUNTNESS (Naming):
   - Function names Max 3 words (e.g., `fetchData` is good; `fetchDataAndProcess` is bad).

OPERATIONAL PROTOCOL:
1. Scan: Read the provided context.
2. Generate: Output WHOLE FILES with the filename in a header.
3. Verify: Ask yourself: "Will Warden reject this?" before printing.
```

---

**License:** MIT
```

**MANUAL ACTION REQUIRED:**
```bash
rm src/main.rs
```
