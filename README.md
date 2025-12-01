
# Warden

**A code quality gatekeeper for AI-assisted development**

Warden creates a feedback loop between your codebase and AI coding assistants. It packages your code with configurable quality constraints, validates AI responses, and only commits changes that pass your rules.

Instead of manually reviewing every AI-generated file, Warden automatically rejects malformed output and asks the AI to try again. When everything passes, it commits and pushes. You stay in flow.

For architectural details, protocol specs, and security philosophy, see [DESIGN.md](DESIGN.md).

## Installation

```bash
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

Verify installation:
```bash
warden --version
```

## Quick Start

**1. Initialize**
Run in your project root to generate a default configuration:
```bash
warden --init
```

**2. Scan**
Check your codebase against the default rules:
```bash
warden
```

**3. Pack for AI**
Compress your codebase into a context-optimized format and copy the path to your clipboard:
```bash
warden pack
```
*Attach the generated file to your AI conversation.*

**4. Apply AI Response**
Copy the AI's response (in the requested Nabla format), then run:
```bash
warden apply
```
Warden validates the response, writes files, runs your configured checks (tests/linters), and commits if everything passes.

## Configuration

Warden is controlled via `warden.toml`. Here are the defaults:

```toml
[rules]
max_file_tokens = 2000              # Files too big to reason about
max_cyclomatic_complexity = 8       # Functions with too many branches
max_nesting_depth = 3               # Deeply nested logic
max_function_args = 5               # Functions doing too many things

# Skip rules entirely for certain files
ignore_tokens_on = [".md", ".lock", ".json"]
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

You can also ignore specific files by adding `.wardenignore` to your root, or adding `// warden:ignore` to the top of a specific source file.

## Commands Reference

| Command | Description |
|---------|-------------|
| `warden` | Scan codebase and report violations |
| `warden --init` | Launch configuration wizard |
| `warden pack` | Generate `context.txt` for AI (copies path to clipboard) |
| `warden apply` | Read AI response from clipboard, validate, and apply |
| `warden check` | Run the test/lint commands defined in config |
| `warden fix` | Run the fix commands defined in config |
| `warden prompt` | Output just the system prompt |
| `warden roadmap` | Manage project tasks (init, show, tasks, audit) |

## Language Support

| Language | Complexity Analysis | Skeleton Mode | Notes |
|----------|:-------------------:|:-------------:|-------|
| Rust | ✅ | ✅ | + `.unwrap()` detection |
| TypeScript | ✅ | ✅ | |
| JavaScript | ✅ | ✅ | |
| Python | ✅ | ✅ | |
| Go | — | — | Token counting only |
| Other | — | — | Token counting only |

## License

MIT — See [LICENSE](LICENSE)
