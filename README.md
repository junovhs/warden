
# 🛡️ Warden Protocol

**High-Integrity Architecture Governance for the AI Age.**

> *"The rules are like the seat belts in a car: Initially, using them is perhaps a little uncomfortable, but after a while, it becomes second nature, and not using them is unimaginable."* — Gerard J. Holzmann

Warden is a structural linter and "Architectural MRI" designed to enforce **NASA/JPL "Power of 10" Rules** in modern software projects.

While originally designed for safety-critical C code, Warden adapts these principles for **Rust, TypeScript, and Python** to solve the "Complexity Creep" and "Context Drift" problems that plague AI-assisted development.

**v0.5.0 Update:** Introducing `warden apply`, the "Spartan Complexity" engine, and "Smart Paranoia".

---

## ⚡ The 3 Laws (Spartan Edition)

Warden uses **Tree-sitter** to parse the AST and enforce strict architectural boundaries. These are not guidelines; they are constraints designed to keep code AI-readable.

### 1. The Law of Atomicity (Holzmann Rule 4)
*   **Rule:** Files must be < **2000 Tokens**.
*   **Rationale:** AI context windows degrade in quality as size increases. Small files ensure deterministic generation and easy verification.
*   **Action:** If a file grows too large, Warden forces a split immediately.

### 2. The Law of Complexity (Holzmann Rules 1 & 2)
*   **Cyclomatic Complexity:** Max **4**. (Standard is 10. Warden is "Spartan").
*   **Nesting Depth:** Max **2**. (No "Arrow Code").
*   **Arity:** Max **5** arguments. (Enforces Data Structures).
*   **Rationale:** *"Simpler control flow translates into stronger capabilities for analysis."* If you cannot read a function in one breath, it is too complex.

### 3. The Law of Paranoia (Smart Safety)
*   **Fallibility:** Functions performing I/O, parsing, or syscalls **MUST** return `Result`.
*   **Infallibility:** Pure logic and UI rendering should rely on the Type System (don't lie to the compiler).
*   **Zero Tolerance:** No `.unwrap()` or `.expect()` allowed. **Ever.**

---

## 🔄 The Cybernetic Workflow

Warden isn't just a linter; it's a closed-loop system for AI development.

### 1. Context (`knit`)
Generates a high-signal context file optimized for LLM ingestion.
```bash
knit --prompt --format xml
```
*   **Smart Filtering:** Strips lockfiles/binaries.
*   **Token Budget:** Calculates exact context usage.
*   **System Prompt:** Bakes the Warden Protocol instructions directly into the context.

### 2. Generation (AI)
You paste the context into your LLM (Claude, GPT-4, Gemini). It generates code.

### 3. Application (`warden apply`)
**Stop copy-pasting files manually.**
Copy the AI's entire response to your clipboard and run:
```bash
warden apply
```
*   **Parses:** Extracts files from XML tags or Markdown blocks.
*   **Validates:** Checks the `<delivery>` manifest against actual files.
*   **Safeguards:** Uses atomic writes and creates backups in `.warden_apply_backup/`.

### 4. Verification (`warden check`)
The "God Command" that runs your language linter AND Warden's structural checks.
```bash
warden check
```

---

## 📸 The Architectural MRI (TUI)

Visualize codebase health in real-time.

```bash
warden --ui
```

<p align="center">
  <img src="assets/screenshot.png" alt="Warden TUI Dashboard" width="700">
</p>

---

## 🛠️ Configuration

Warden works for **Polyglot** projects. It acts as the General, delegating syntax checks to language-specific Lieutenants.

Run `warden --init` to generate a `warden.toml`.

**For Web/TypeScript (using Biome):**
```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10 # TS usually needs more breathing room than Rust
max_nesting_depth = 4

[commands]
# Warden runs this first. If it fails, Warden fails.
check = "npx @biomejs/biome check src/"
```

**For Rust (Strict):**
```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 4
max_nesting_depth = 2

[commands]
check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
```

---

## 📦 Installation

```bash
cargo install --path . --force
```

**License:** MIT
