# pm_encoder: The Technical Blueprint
## From Context Compression to AI Collaboration Infrastructure

**Status:** Living Document | **Focus:** Reference Implementation (Python)

## Executive Summary
pm_encoder is evolving from a serialization script into **AI Collaboration Infrastructure**. As context windows grow, the problem shifts from "what fits" to "what matters." This document outlines the architectural vision for the Python Reference Implementation.

## Core Philosophy
1.  **Context is the new Compilation:** We translate codebases into AI-consumable formats.
2.  **Intent beats Syntax:** Users declare *what* they want (Lenses), not *how* to get it (Flags).
3.  **Reference First:** The Python version (`pm_encoder.py`) is the source of truth for all logic, patterns, and behaviors.

## Architecture: The Shared Grammar
To support future scale (and potential ports to Rust/WASM), logic must be **Declarative**.
*   **Current:** Hardcoded Python Regex in `LanguageAnalyzer` classes.
*   **Future:** JSON-defined patterns in `.pm_encoder_config.json`.
    *   *Benefit:* A single config defines how to parse a language for any engine.

## Roadmap

### Phase 1: Foundation (Current)
*   **v1.2.x**: Context Lenses, Structure Mode, Native Rust Support.
*   **Goal**: Establish the "Context Engineer" UX.

### Phase 2: Scale & Preview (v1.3.0)
*   **Streaming Output**: Support for 100k+ file repositories via generators.
*   **Interactive Mode**: CLI wizard for lens selection.
*   **Goal**: Instant feedback (TTFB ~0ms) for large repos.

### Phase 3: The Declarative Shift (v1.4.0)
*   **JSON Pattern Engine**: Move regex logic out of Python code and into Config.
*   **Community Plugins**: Allow users to add languages via JSON config, not just Python code.
*   **Goal**: Universal language support without code changes.

## Contribution Guide
We prioritize:
1.  **Zero Dependencies**: Standard Library only.
2.  **Backward Compatibility**: The Plus/Minus format is sacred.
3.  **Test Coverage**: All features must pass `tests/test_pm_encoder.py`.
