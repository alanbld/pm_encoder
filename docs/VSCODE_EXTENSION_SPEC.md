# pm_encoder VS Code Extension - UX Specification

**Version:** 0.1.0 (Draft)
**Target:** v2.1 WASM Release
**Status:** Planning

---

## Executive Summary

A VS Code extension that provides **one-click, token-budgeted context generation** powered by the pm_encoder WASM engine. Code never leaves the machine.

---

## 1. Core Commands

### Primary Command: Copy Context

```
Command Palette: "pm_encoder: Copy Context"
Keyboard Shortcut: Cmd+Shift+E (Mac) / Ctrl+Shift+E (Windows/Linux)
```

**Behavior:**
1. Reads all files in workspace (respecting .gitignore)
2. Applies configured lens and token budget
3. Copies serialized context to clipboard
4. Shows notification: "Context copied (847 files, 98.2k tokens)"

**Output:** Plus/Minus format, ready to paste into Claude/GPT/Gemini

---

### Secondary Commands

| Command | Description | Output |
|---------|-------------|--------|
| `pm_encoder: Copy Context` | Full budgeted context | Clipboard |
| `pm_encoder: Copy Selection` | Context for selected files only | Clipboard |
| `pm_encoder: Preview Context` | Open in side panel (read-only) | Editor panel |
| `pm_encoder: Save Context` | Save to CONTEXT.txt | File |
| `pm_encoder: Generate Init Prompt` | Create CLAUDE.md + CONTEXT.txt | Files |

---

## 2. Status Bar Integration

```
┌─────────────────────────────────────────────────────────────────┐
│  [pm_encoder: 847 files | 98.2k tokens | architecture]  [⚙️]   │
└─────────────────────────────────────────────────────────────────┘
```

**Click behavior:**
- Left click: Run "Copy Context"
- Right click: Open settings

**Live updates:**
- File count updates on workspace change
- Token estimate recalculated on save (debounced 2s)

---

## 3. Settings Panel

Accessible via:
- Status bar gear icon
- Command Palette: "pm_encoder: Settings"
- VS Code Settings UI (under Extensions > pm_encoder)

### Configuration Options

```json
{
  "pm_encoder.lens": "architecture",
  "pm_encoder.tokenBudget": "100k",
  "pm_encoder.budgetStrategy": "hybrid",
  "pm_encoder.excludePatterns": [
    "node_modules",
    "*.lock",
    "dist"
  ],
  "pm_encoder.includePatterns": [],
  "pm_encoder.showStatusBar": true,
  "pm_encoder.autoUpdateEstimate": true
}
```

### Settings UI

```
┌─────────────────────────────────────────────────────────────────┐
│  pm_encoder Settings                                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Lens          [architecture ▼]                                 │
│                ├── architecture (signatures, config)            │
│                ├── debug (recent changes, full content)         │
│                ├── security (auth, crypto, validation)          │
│                └── minimal (entry points only)                  │
│                                                                 │
│  Token Budget  [100k        ] tokens                            │
│                Shorthand: 50k, 100k, 200k, 1M, 2M               │
│                                                                 │
│  Strategy      [hybrid ▼]                                       │
│                ├── drop (skip files that don't fit)             │
│                ├── truncate (structure mode for large files)    │
│                └── hybrid (auto-truncate >10%, then drop)       │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Exclude Patterns                                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ node_modules                                             │   │
│  │ *.lock                                                   │   │
│  │ dist                                                     │   │
│  │ .git                                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│  [+ Add Pattern]                                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Preview Panel

Command: `pm_encoder: Preview Context`

```
┌─────────────────────────────────────────────────────────────────┐
│  pm_encoder Preview                                    [Copy]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ++++++++++ src/main.rs ++++++++++                              │
│  use std::env;                                                  │
│  use pm_encoder::{serialize_project, EncoderConfig};            │
│                                                                 │
│  fn main() {                                                    │
│      let args: Vec<String> = env::args().collect();             │
│      ...                                                        │
│  }                                                              │
│  ---------- src/main.rs a7b3c9d2... ----------                  │
│                                                                 │
│  ++++++++++ src/lib.rs [TRUNCATED: 1,247 lines] ++++++++++      │
│  pub mod analyzers;                                             │
│  pub mod budgeting;                                             │
│  ...                                                            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  Files: 47 | Tokens: 98,234 | Truncated: 12 | Dropped: 3        │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**
- Syntax highlighting for Plus/Minus format
- Collapsible file sections
- Click file header to open source file
- Footer shows budget usage stats

---

## 5. Context Menu Integration

Right-click on file/folder in Explorer:

```
┌─────────────────────────────┐
│  pm_encoder                 │
│  ├── Copy Context (this)    │
│  ├── Add to Include         │
│  └── Add to Exclude         │
└─────────────────────────────┘
```

---

## 6. Notifications

### Success States

```
✓ Context copied (847 files, 98.2k tokens)
  [Open Preview]  [Dismiss]
```

```
✓ CLAUDE.md + CONTEXT.txt generated
  [Open CLAUDE.md]  [Dismiss]
```

### Warning States

```
⚠ Budget exceeded - 23 files dropped (hybrid strategy)
  [Show Dropped]  [Increase Budget]  [Dismiss]
```

```
⚠ No files matched include patterns
  [Open Settings]  [Dismiss]
```

### Error States

```
✗ Failed to read workspace
  [View Error]  [Dismiss]
```

---

## 7. Keyboard Shortcuts

| Shortcut | Command |
|----------|---------|
| `Cmd+Shift+E` | Copy Context |
| `Cmd+Shift+Alt+E` | Preview Context |
| `Cmd+Shift+I` | Generate Init Prompt |

(All configurable via VS Code keybindings)

---

## 8. Walkthrough / Onboarding

First-time users see a walkthrough:

```
┌─────────────────────────────────────────────────────────────────┐
│  Welcome to pm_encoder                                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1️⃣  Copy Context                                               │
│      Press Cmd+Shift+E to copy your project context             │
│      to clipboard, ready for Claude or GPT.                     │
│                                                                 │
│  2️⃣  Set Token Budget                                           │
│      Click the status bar to configure your token limit.        │
│      Default: 100k tokens (fits Claude's context).              │
│                                                                 │
│  3️⃣  Choose a Lens                                              │
│      "architecture" shows signatures only.                      │
│      "debug" shows recent changes with full content.            │
│                                                                 │
│                                           [Get Started]         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    VS Code Extension (TypeScript)               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  extension.ts                                             │  │
│  │  - Command registration                                   │  │
│  │  - Status bar management                                  │  │
│  │  - Settings integration                                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                              ▼                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  WASM Bridge (wasm-bindgen)                               │  │
│  │  - Load pm_encoder.wasm                                   │  │
│  │  - Convert VS Code file API → (path, content) pairs       │  │
│  │  - Call ContextEngine.generate_context()                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                              ▼                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  pm_encoder.wasm (~400KB)                                 │  │
│  │  - ContextEngine (pure functions)                         │  │
│  │  - LensManager                                            │  │
│  │  - Token budgeting                                        │  │
│  │  - No I/O (files passed in as strings)                    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 10. What's NOT in v2.1 (Scope Control)

To ship fast, these are explicitly **out of scope** for v2.1:

| Feature | Reason | Future Version |
|---------|--------|----------------|
| Real-time file watching | Complexity, battery drain | v2.2+ |
| Custom lens editor | Settings UI sufficient | v2.3+ |
| Multi-workspace support | Edge case | v2.2+ |
| Remote SSH/WSL files | WASM can't access remote FS | Never (use CLI) |
| Auto-paste to Claude | Security concerns | Never |
| Diff view (before/after) | Nice-to-have | v2.2+ |

---

## 11. Success Criteria (MVP)

- [ ] "Copy Context" works in <50ms for 1000-file project
- [ ] Status bar shows accurate file/token count
- [ ] Settings persist across sessions
- [ ] WASM bundle <500KB
- [ ] Works offline (no network calls)
- [ ] Published to VS Code Marketplace
- [ ] 4.5+ star rating after 100 reviews

---

## 12. Open Questions

1. **Lens presets:** Should we bundle all 4 lenses or allow custom lens files?
2. **Token counting:** Use tiktoken WASM or heuristic (4 chars = 1 token)?
3. **Large workspaces:** Show progress indicator for >5000 files?
4. **Telemetry:** Opt-in anonymous usage stats?

---

*Document Version: 0.1.0*
*Created: 2025-12-19*
*Author: pm_encoder Core Team*
