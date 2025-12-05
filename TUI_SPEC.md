# SlopChop Mission Control: Design Spec

**Objective:** Replace the fragmented CLI workflow with a persistent, unified Terminal User Interface (TUI).

---

## 1. The Layout ("The Cockpit")

We use a standard "Holy Grail" layout, heavily inspired by NASA control consoles and sci-fi dashboards (Alien/Nostromo).

```text
+---------------------------------------------------------------+
|  SLOPCHOP v0.8.0  |  DIRTY  |  ??? 100%  |  TOKEN BUDGET: 45%  |  <-- Header
+-------------------+-------------------------------------------+
| [1] ROADMAP       |                                           |
| [2] CHECKS        |  Active Tab Content (Dynamic Pane)        |
| [3] CONTEXT       |                                           |
| [4] CONFIG        |                                           |
| [5] LOGS          |                                           |
|                   |                                           |
|                   |                                           |
|                   |                                           |
+-------------------+-------------------------------------------+
| > Waiting for clipboard input... [CTRL+V to Apply]            |  <-- Status Bar
+---------------------------------------------------------------+
```

### The Header (Telemetry)
- **Version**: Current build.
- **Git Status**: CLEAN/DIRTY (updates live).
- **Health**: Logic Integrity Score (derived from config strictness).
- **Budget**: Context window usage estimation.

### The Sidebar (Navigation)
Persistent navigation. Number keys `1-5` switch tabs instantly.

---

## 2. The Views

### [1] ROADMAP (The Flight Plan)
*Interactive Task Manager*
- **Visuals**: Tree view of `ROADMAP.md`.
- **Actions**:
    - `SPACE`: Toggle check/uncheck.
    - `ENTER`: Focus on a task (sets it as current context).
    - `a`: Add new item (modal input).
- **Goal**: You never edit `ROADMAP.md` by hand again.

### [2] CHECKS (Diagnostics)
*Live Linter & Test Runner*
- **Visuals**: Split pane. Left: List of suites (Clippy, Test, SlopChop). Right: Output stream.
- **Actions**:
    - `r`: Run all checks.
    - `f`: Fix auto-fixable issues.
- **Innovation**: Streaming output. No more scrolling history. The logs stay in the box.

### [3] CONTEXT (The Payload)
*Packer Visualization*
- **Visuals**: File tree with size heatmaps (Red = Huge, Green = Tiny).
- **Actions**:
    - `SPACE`: Toggle file inclusion for next prompt.
    - `s`: Toggle skeletonization for selected file.
    - `c`: Copy context to clipboard.
- **Metric**: Real-time token count as you toggle files.

### [4] CONFIG (Systems)
*Existing Config TUI*
- We port the existing configuration screen into this tab.

### [5] LOGS (The Black Box)
*Daemon Output*
- Raw feed of what the background daemon/watcher is doing.
- History of applied patches.

---

## 3. The "Watch Mode" Integration

The TUI is not just a viewer; it is the **Daemon**.

1.  **Clipboard Monitoring**: The TUI runs the clipboard watcher in a background thread.
2.  **Interrupt Handling**: When you copy code from Claude/GPT:
    - The TUI flashes/notifies "PAYLOAD RECEIVED".
    - A modal pops up: "Apply Patch? [Y/n] [d]iff".
    - `d` shows a diff view.
    - `y` applies it, runs checks, and updates the [2] CHECKS tab results live.

---

## 4. Implementation Strategy

### Phase 1: The Shell
- `slopchop dashboard` command.
- Basic Ratatui layout implementation.
- Tab switching logic.

### Phase 2: Migration
- Port `ROADMAP` viewer into Tab 1.
- Port `Config` editor into Tab 4.

### Phase 3: The Runner
- Implement the async `CHECKS` runner (Tab 2).
- Redirect stdout/stderr from `cargo test` into the TUI buffer.

### Phase 4: The Loop
- Integrate Clipboard Watcher.
- Implement the "Popup" system for incoming patches.

---

*Verified by SlopChop Protocol.*