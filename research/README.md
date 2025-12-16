# The Twins: A Research Project

**Investigating TDD-Driven Convergence in Dual-Engine Development**

## Overview

This research project tracks the development of `pm_encoder`, a dual-engine codebase with:
- **Python Engine** (reference implementation, v1.6.0) - Streaming, 95% coverage
- **Rust Engine** (performance implementation, v0.5.0) - Streaming, 85% coverage

The goal is to empirically validate the **Convergence Hypothesis**: that Test-Driven Development using test vectors can accelerate feature parity between two language implementations 3-4x faster than traditional development.

**Result:** ✅ **VALIDATED** - 100% parity achieved in just 3 days (vs. projected 3.5 months).

## Research Questions

1. **Speed**: How much faster is TDD-driven parity development vs traditional parallel implementation?
2. **Quality**: Does the test vector contract improve code quality in both engines?
3. **Sustainability**: Can this pace be maintained over 6+ months?
4. **Scalability**: Does the approach scale to complex features (analyzers, lenses)?
5. **Architecture vs Language** (RQ5): Can superior architecture overcome language speed differences? ✅ *Answered*
6. **Complexity vs Density** (RQ6): Does lower cyclomatic complexity correlate with maintainability, even at lower semantic density?

## Current Status (Dec 16, 2025)

### 🎉 CONVERGENCE ACHIEVED

**On December 16, 2025, both engines reached 100% test vector parity.**

### Parity Metrics
- **Test Vectors:** 25/25 passing (100% parity) ✅
- **Python Coverage:** 95%
- **Rust Coverage:** 85.38%
- **Timeline:** 15+ days ahead of original schedule
- **Streaming Parity:** Both engines support `--stream` flag

### Milestones Completed
- ✅ v0.1.0 - Foundation (Rust)
- ✅ v0.2.0 - Core Serialization (Rust)
- ✅ v0.3.0 - Config System (Rust, 80% parity)
- ✅ v0.4.0 - Serialization Vectors (100% parity)
- ✅ v1.5.0 - Rust Parity & Interface Protocol (Python)
- ✅ v1.6.0 - The Streaming Pipeline (Python)
- ✅ v0.5.0 - Streaming Parity (Rust)

---

## Phase 2.5: The Streaming Convergence (v1.6.0 / v0.5.0)

**Date:** December 16, 2025

### Observation
Both engines now support **Streaming Architecture**:
- Python v1.6.0: Generators (`yield`)
- Rust v0.5.0: Iterators (`impl Iterator`)

### Research Question (RQ5) - Answered
> Can an interpreted language with superior architecture (Streaming) beat a compiled language with inferior architecture (Batch) on Time-To-First-Byte (TTFB)?

**Answer:** Yes! Python streaming (88ms) matched Rust batch (~1.6s). But when Rust adopts streaming (5ms), it dominates.

### Final Results (React repo, ~6,905 files)

| Engine | Version | Architecture | TTFB |
|--------|---------|-------------|------|
| **Rust** | **v0.5.0** | **Streaming** | **5ms** |
| Python | v1.6.0 | Streaming | 88ms |
| Rust | v0.5.0 | Batch | 1,600ms |
| Python | v1.6.0 | Batch | 3,900ms |

### Key Findings
1. **Architecture Matters:** Streaming reduces TTFB by 320x (Rust) / 44x (Python)
2. **Language Matters Too:** Rust streaming is 17x faster than Python streaming
3. **Synergy Effect:** Combining superior architecture + compiled speed creates exponential gains

### Status
- **Python v1.6.0**: Streaming implemented ✅
- **Rust v0.5.0**: Streaming implemented ✅
- **Parity**: Restored (both engines support `--stream` flag)

---

## Phase 3: Complexity Analysis (v1.6.0 vs v0.5.0)

**Date:** December 16, 2025
**Tool:** [lizard](https://github.com/terryyin/lizard) v1.19.0

### Hypothesis

> Rust will show higher Semantic Density (Tokens/NLOC) due to its expressive type system.

### Result: ❌ REJECTED

Python is **7% denser** (7.39 vs 6.89 tokens/line).

### New Discovery: The Pattern Dividend

Rust shows **50% Lower Cyclomatic Complexity** (Avg CCN 3.33 vs 6.72).

### Metrics

| Metric | Python | Rust | Delta |
|--------|--------|------|-------|
| NLOC (Lines of Code) | 1,257 | 1,497 | **+19%** |
| Function Count | 50 | 76 | **+52%** |
| Avg Cyclomatic Complexity | 6.72 | 3.33 | **-50%** |
| Max Cyclomatic Complexity | 22 | 35 | +59% |
| Semantic Density (Tok/NLOC) | 7.39 | 6.89 | -7% |

### Interpretation

1. **The Type Tax**: Rust's explicit typing increases token count without adding "meaning," lowering semantic density. Every `&str`, `Vec<String>`, and `Result<T, E>` adds tokens.

2. **The Pattern Dividend**: Rust's pattern matching (`match`, `if let`, `?` operator) replaces complex Python `if/elif/else` chains, **halving the cyclomatic complexity**.

3. **The Decomposition Effect**: Rust has 52% more functions (76 vs 50), forcing better architectural decomposition. Complexity is spread across smaller units.

4. **The Outlier Problem**: While *average* CCN is lower in Rust, the *max* CCN is higher (35 vs 22). The function `truncate_structure` needs refactoring.

### Highest Complexity Functions

| Rank | Python | CCN | Rust | CCN |
|------|--------|-----|------|-----|
| 1 | `analyze_lines` | 22 | `truncate_structure` | 35 |
| 2 | `serialize` | 16 | `main` (CLI) | 17 |
| 3 | `collect_files_generator` | 13 | `truncate_smart` | 13 |

### Architectural Conclusion

> The Rust engine is **larger** (19% more LOC) but **significantly simpler to reason about locally** (50% lower avg CCN).

This suggests a trade-off:
- **Python**: Compact but complex functions (higher cognitive load per function)
- **Rust**: Verbose but simple functions (lower cognitive load, more files to navigate)

### Research Question (RQ6) - Open

> Does lower average CCN correlate with better maintainability, even when semantic density is lower?

This requires longitudinal study: tracking bug rates, refactoring frequency, and developer onboarding time over the next 6 months.

### Data

Raw metrics stored in [`data/complexity.json`](./data/complexity.json).

---

## Methodology

See [METHODOLOGY.md](./METHODOLOGY.md) for detailed research approach.

## Data Collection

Daily snapshots tracked in `data/daily_snapshots.csv`:
- Python test coverage
- Rust test coverage
- Test vector parity percentage
- Lines of code (both engines)
- Velocity metrics

## Timeline

- **Start:** December 13, 2025
- **Convergence:** December 16, 2025 (100% parity achieved!)
- **Duration:** Ongoing research through June 2026
- **Original Target:** 100% parity by March 31, 2026 - **ACHIEVED 3.5 MONTHS EARLY!**

## Publications

Results will be published as:
1. Blog series documenting the journey
2. Academic paper on TDD acceleration
3. Open-source case study

## Contributing

This is an active research project. See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

## License

Research data and findings: CC BY 4.0
Code: MIT License (see [LICENSE](../LICENSE))

---

**Last Updated:** December 16, 2025
**Researchers:** Multi-AI Development Team (Opus Architect, Sonnet Orchestrator, Claude Code)
