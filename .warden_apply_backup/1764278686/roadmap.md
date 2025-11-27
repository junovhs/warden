# Warden Protocol Roadmap

## Current State: v0.4.0 ✓

The core loop works:
- Generate context with `knit --prompt`
- Chat with AI
- Apply responses with `warden apply`
- Verify with `warden` and `warden check`
- **Self-hosting:** Warden enforces its own rules on its own codebase

---

## v0.5.0 — Bulletproof Apply

**Theme:** If it applies, it's valid. If it's invalid, it rejects hard.

### Validation Hardening

- [x] **Truncation detection (smart)**  
  Reject files that are obviously incomplete:
  - [x] Truncation markers: `// ...`, `/* ... */`, `// rest of file`, `// etc`
  - [ ] Unbalanced braces/brackets (deferring to v0.6 for robustness)
  - [ ] Files that end mid-statement (deferring)
  
  *Goal: Zero false positives. If Warden rejects it, it was definitely broken.*

- [x] **Path safety validation**  
  Block dangerous paths before they touch disk:
  - `../` directory traversal
  - Absolute paths (`/etc/passwd`, `C:\Windows\...`)
  - Sensitive targets: `.git/`, `.env`, `.ssh/`, `id_rsa`, `.aws/`, `credentials`
  - Hidden files starting with `.` (configurable)
  
  *Enterprise-grade paranoia.*

- [x] **Strict format enforcement**  
  If AI doesn't use `<file path="...">` tags, reject immediately with clear error message explaining the required format. No fallback parsing. No guessing. Garbage in = garbage out.

- [x] **Markdown block rejection**  
  Rejects files containing fenced code blocks to prevent AI formatting artifacts from corrupting source.

- [x] **Robust Delimiter Protocol (The "Nabla" Format)**
  Replace fragile XML tags with high-entropy Unicode fences to prevent Markdown rendering issues and AI confusion.
  - Start: `∇∇∇ path/to/file.rs ∇∇∇`
  - End:   `∆∆∆`
  - *Prevents chat interfaces from hiding tags or interpreting them as HTML.*

### Workflow Enhancement (v0.5.0)

- [x] **Error injection in knit**  
  When `knit --prompt` runs, it scans the files being packed. If violations exist, they are appended to the context header.
  *AI sees what's broken. AI fixes it.*

- [x] **Smart Clipboard Protocol**
  - Auto-detect content size.
  - If < 1500 tokens: Copy as raw text.
  - If > 1500 tokens: Save to temp file, copy *file handle* to clipboard.
  - **The Garbage Man:** Auto-cleanup temp files older than 15 mins.

- [ ] **The "Plan" Protocol** (Prompt Update)
  Update system prompt to enforce a `<plan>` block before `<delivery>`.
  - AI must explain *why* it is making changes in natural language first.
  - `warden apply` extracts the plan and displays it to the user for confirmation before writing files.
  - *Mitigates "coding without thinking".*

### Git Integration (Experimental)

- [ ] **warden apply --commit**  
  On successful apply:
  1. `git add .`
  2. Auto-generate commit message from the `<plan>` block or `<delivery>` manifest
  3. Commit (no push by default)

- [ ] **warden apply --commit --push**  
  Same as above, but also pushes.

*Philosophy: If it passes validation, commit it. Use git as your undo. Atomic commits per apply.*

### Implemented (Keep for Now)

- [x] **Backup system** — Creates `.warden_apply_backup/TIMESTAMP/` before writes. Simple insurance until git workflow is muscle memory.

---

## v0.6.0 — Context Intelligence (The Saccade Merge)

**Theme:** The "Map vs. Territory" Architecture. Solve the "Lost in the Middle" problem.

### The Skeletonizer (Ported from Saccade)
- [ ] **Port `parser.rs` from Saccade**
  - Integrate Tree-sitter-based stripping of function bodies.
  - Keep structs, enums, trait signatures, and function signatures.
  - Goal: Reduce file size by ~70-90% while retaining API visibility.
  
- [ ] **knit --skeleton**
  - Generates a context file where *every* file is skeletonized.
  - Useful for "high level architectural planning" with the AI.

### Smart Knitting (Context Slicing)
- [ ] **Dependency Graphing (Saccade Stage 1)**
  - Implement Tree-sitter queries to find `mod`, `use`, `import`, and `require`.
  - Build a lightweight graph of local file dependencies.

- [ ] **knit src/main.rs --smart**
  - **The Territory:** Includes full source code of `src/main.rs` and its *immediate* imports.
  - **The Map:** Includes *skeletons* of the rest of the project (or at least the rest of the module).
  - *Result:* AI has deep focus on the task, broad awareness of the project, but low token count.

### The "Generate-Then-Structure" Workflow
- [ ] **Decoupled Reasoning**
  - Update `warden apply` to handle a two-step generation process if we move to an agentic loop later.
  - Step 1: Generate Plan (Natural Language).
  - Step 2: Generate Code (Strict XML).
  - *Reduces the cognitive load of formatting on the AI's reasoning capabilities.*

---

## v0.7.0 — Verification & Safety

**Theme:** Trust the tool, verify the AI.

### Property-Based Testing (The Dream)
- [ ] **warden gen-test <file>**
  - Uses AI to write *Property-Based Tests* (`proptest` for Rust, `hypothesis` for Python).
  - Prompt: "Analyze this code. Write a property test that asserts invariants. Do not write unit tests."
  - Automatically saves to `tests/warden_props_<name>.rs`.
  - *Moves verification from "it compiles" to "it is mathematically sound".*

### Smarter Analysis (Refined)

- [ ] **Function-level violation reporting**  
  Not just "file has violations" but detailed breakdown:
  
  *Learn from the patterns. Understand WHY it's complex.*

- [ ] **Incremental scanning**  
  Only re-analyze changed files:
  - Track file mtimes in `.warden_cache`
  - Or use `git status` to find modified files
  - Full rescan on config change

---

## v0.8.0 — Ecosystem & Polish

**Theme:** CI/CD and tooling integration.

- [ ] **Test suite**
  - Unit tests for each module
  - Integration tests: knit → apply → verify flow

- [ ] **Performance benchmarks**
  - Scan time vs file count

- [ ] **CLI stability guarantee**
  - Document all flags and subcommands

---

## v1.0.0 — Release

- [ ] Published to **crates.io**
- [ ] **Homebrew**
- [ ] **Scoop/Winget**

---

## Principles

1. **Reject bad input, don't fix it**  
   Warden is a gatekeeper, not a fixer.

2. **Git is the undo system**  
   Don't reinvent version control.

3. **Explicit > Magic**  
   If AI doesn't follow the format, fail loudly.

4. **Learn from violations**  
   Error messages should teach, not just complain.

5. **Eat your own dogfood**  
   Warden must pass its own rules.