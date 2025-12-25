---
# Product Requirements Document (PRD)
**voyager-ast v1 – The Structural Optics Layer for Voyager Observatory**

## 1. Problem Statement

Voyager Observatory needs a fast, resilient, multi-language structural indexer to power its Fractal Protocol, Planetarium View, and Microscope View. Current regex-based parsing is brittle, shallow, and hard to extend beyond a handful of languages. VO requires a dedicated "optics" subsystem that:

- Provides a **file-based structural model** (functions, blocks, imports, control flow, comments) across 60–100 languages.
- Recovers gracefully from syntax errors and incomplete code.
- Integrates cleanly with Tree-sitter while exposing a **language-agnostic IR** that higher layers can rely on.

Without this, VO cannot scale to large, polyglot repositories or deliver reliable, intent-driven exploration to LLMs and users.

***

## 2. Goals and Objectives

**Primary goals**

- Deliver a **Rust crate, `voyager-ast`**, that:
  - Parses source files via Tree-sitter.
  - Normalizes them into a stable, language-agnostic IR.
  - Supports two operating modes: **Index (Planetarium)** and **Zoom (Microscope)**.

- Achieve "**telescope, not compiler**" behavior:
  - ~90% structural correctness for targeted constructs.
  - Best-effort recovery with explicit "unknown" regions.

**Quantitative objectives**

- Support **≥15 languages** in v1, including the Core Fleet: Rust, Python, TypeScript, ABL.
- Index mode:
  - Handle **100k LOC** polyglot repos in **≤ 3 seconds** on a typical developer machine.
- Zoom mode:
  - Per-file zoom (core fleet) in **≤ 50 ms** median, **≤ 200 ms** p95.
- Error resilience:
  - For intentionally corrupted test corpora, return a non-empty IR tree for **≥ 98%** of files, with Unknown/Unparsed nodes as needed.

***

## 3. User Personas

**1. VO Engine Developer (Internal)**
- Motivation: Stable, predictable optics layer to build VO's Fractal Protocol and lenses.
- Pain points: Regex hacks, inconsistent structures across languages, difficulty scaling language support.

**2. Tool Builder / OSS Contributor (External)**
- Motivation: Use voyager-ast as a drop-in structural indexer for their own CLIs, LSP servers, analyzers.
- Pain points: Existing parsers are language-specific, hard to integrate, or require deep compiler knowledge.

**3. AI-augmented Developer (Indirect)**
- Motivation: Reliable, structured "mission logs" and navigation for LLM-based tools.
- Pain points: Ad-hoc file concatenation, shallow context, brittle manual selection.

***

## 4. Use Cases

1. **Planetarium Scan (Index Project)**
   - VO or a third-party tool calls `index_project(root, options)`.
   - voyager-ast:
     - Detects file languages.
     - Parses each file lazily in Index mode.
     - Returns a **PlanetariumModel** with:
       - Files, top-level declarations, imports, comments.

2. **Fractal Zoom on a Function**
   - User or LLM requests deeper understanding of a specific function.
   - VO translates that to `zoom_into(file_id, symbol_id, options)`.
   - voyager-ast:
     - Re-parses or deepens the parse for that file.
     - Returns a **MicroscopeModel**:
       - Full Block, ControlFlow, Calls, detailed comments for that region.

3. **Resilient Parsing of Broken Code**
   - VO indexes a WIP branch with syntax errors.
   - voyager-ast:
     - Still returns a tree.
     - Flags uncertain regions as `UnknownNode` or `UnparsedBlock`.
   - VO can surface this "dark matter" to the LLM instead of silently dropping content.

4. **Third-party Language Adapter Contribution**
   - Contributor adds support for a new language with a Tree-sitter grammar.
   - Implements mapping trait for:
     - Declarations, imports, comments (Index mode).
     - Blocks, control flow, calls (Zoom mode).
   - Runs visualization/debugging tools to validate mapping.

***

## 5. Key Features

1. **File-based Structural IR**

   - Primary unit: **File tree**.
   - Core IR types (v1, stable):
     - `File`
     - `Region` (span, language id)
     - `Declaration` (function, method, class, type, constant)
     - `Block`
     - `ControlFlow` (if/else, loop, switch/match)
     - `Call` (name, callee text; best-effort)
     - `ImportLike` (import/require/include/using/module ref)
     - `Comment` (line/block, with best-effort attachment to nearby nodes)
     - `UnknownNode`, `UnparsedBlock` for fuzzy regions.

2. **Two Operating Modes: Index & Zoom**

   - `index_project(root, options) -> PlanetariumModel`
     - Top-level declarations, imports, file-level comments per file.
     - No intra-function control-flow by default.
   - `zoom_into(file_id, symbol_id, options) -> MicroscopeModel`
     - Full body of target symbol: nested blocks, control flow, calls, comments.

3. **Tree-sitter Integration (Lazy, Chunked)**

   - Use Tree-sitter for parsing each file.
   - Lazy parsing strategy:
     - Only parse a file when needed (first Index or Zoom access).
     - Cache parse trees where appropriate in the process lifetime.
   - "Chunked" approach to avoid unnecessary deep walking in Index mode.

4. **Error-tolerant, Explicit Fuzziness**

   - Always attempt to return a tree:
     - On syntax errors, insert `UnknownNode` / `UnparsedBlock` with spans.
   - No "silent drop" of malformed regions.
   - VO can use this to signal uncertainty to LLMs.

5. **Language Adapters via Plugin Registry**

   - Single crate in v1 with a **registry of adapters**:
     - Core Fleet: grammar-level mapping.
     - Long tail: pattern-based mapping on node types / Tree-sitter queries.
   - Designed so adapters can later be extracted to `voyager-ast-lang-*` crates.

6. **Deterministic, Syntactic-only Core**

   - voyager-ast:
     - No semantic overlays.
     - No learning or heuristic changes based on LLM feedback.
   - Semantic tags (domains, "Bright Stars") live adjacent to IR, not inside it.

***

## 6. Success Metrics

- **Adoption & Stability**
  - Used as the **sole structural backend** for VO in production (no regex fallbacks).
  - Zero "showstopper" parsing regressions in VO across 3 consecutive releases.

- **Performance**
  - Meet the latency targets in Goals & Objectives for Index and Zoom.
  - Memory profile remains stable and bounded per file (no unbounded parse forests).

- **Coverage**
  - Core Fleet: Rust, Python, TypeScript, ABL at "high-resolution" (Zoom fully supported).
  - At least 10 additional languages in Index mode (C, C++, Java, Go, PHP, Ruby, etc.).

- **Contributor Ergonomics**
  - Documented "Add a Language in N Steps" guide.
  - At least 2 external PRs adding or improving language support within 3–6 months of open sourcing.

***

## 7. Assumptions

- Tree-sitter grammars exist and are mature enough for the core target languages.
- VO's current and near-term use is **read-only** over repo checkouts (no live buffer edits).
- LLMs consume **narrative summaries** generated from voyager-ast output, not raw ASTs.
- Cross-file relations (imports → definitions, calls → targets) can remain best-effort/name-based in v1; no full symbol resolution required.

***

## 8. Timeline

**Phase 1A – Core Skeleton (2–3 weeks)**
- Define IR types and traits.
- Integrate Tree-sitter runtime.
- Implement Index/Zoom API surfaces and basic Rust adapter.

**Phase 1B – Core Fleet Support (4–6 weeks)**
- Implement adapters for Rust, Python, TypeScript, ABL.
- Add Unknown/Unparsed behavior and fuzziness handling.
- Benchmarks and perf tuning for CLI/MCP workloads.

**Phase 1C – Long-tail Languages + Ergonomics (4 weeks)**
- Add 5–10 additional languages in Index mode.
- Build visualization/debugging utilities (e.g., dump IR to JSON, AST overlay viewer).
- Author contribution guides and internal docs.

**Phase 2 – Extraction & Hardening (post-v1)**
- Optionally split language adapters into `voyager-ast-lang-*` crates.
- Add incremental parsing hooks for future live-edit scenarios.
- Integrate versioning contracts for VO and external consumers.

***

## 9. Stakeholders

- **Product / Architecture**
  - Chief Architect, Voyager Observatory (decision owner for IR and APIs).
- **Engineering**
  - Core Rust team (voyager-ast implementation).
  - VO engine team (integration & Fractal Protocol).
- **Developer Relations / Community**
  - Maintains documentation, guides, and handles external contributions.
- **Research / AI**
  - Defines how IR is transformed into narratives and Fractal context for LLMs.

---

## 10. Known Constraints or Dependencies

- Dependence on **Tree-sitter grammar quality**; some languages may be noisy or incomplete.
- Performance constraints on very large monorepos; may require future caching/indexing strategies.
- Need to maintain **deterministic output**, as VO depends on repeatable behavior for mission logs and tests.
- ABL and other niche languages may require custom or immature grammars, increasing adapter complexity.

***

## Open Questions

- How should **PlanetariumModel** and **MicroscopeModel** be serialized for external consumers?
  - JSON only, or also binary (e.g., MessagePack) for performance?
- Do we need an **experiment flag** system in voyager-ast (for new heuristics) that VO can toggle per deployment?
- What is the minimum level of **comment-to-node attachment** required (nearest node, enclosing node, custom strategies)?

***

## Risks and Mitigations

- **Risk: Over-complex IR in v1.**
  - Mitigation: Freeze the small v1 IR (File, Region, Declaration, Block, ControlFlow, Call, ImportLike, Comment, Unknown/Unparsed). Defer richer semantics to later versions.

- **Risk: Tree-sitter inconsistencies across languages.**
  - Mitigation: For long-tail languages, restrict to Index mode; treat Zoom as "experimental" until coverage is validated.

- **Risk: Performance regressions as languages are added.**
  - Mitigation: Continuous benchmarks on representative repos; enforce budget per file and per-project in CI.

- **Risk: External users depend on unstable APIs.**
  - Mitigation: Version APIs clearly (e.g., `v1` module), document stability guarantees, mark experimental parts.

---

## Design Philosophy: Telescope, Not Compiler

**voyager-ast** is explicitly designed as a "telescope" – an observation instrument – not a compiler or semantic analyzer. This philosophy manifests in several key design decisions:

1. **Best-Effort Recovery Over Formal Correctness**
   - When encountering syntax errors, we insert `UnknownNode` markers rather than failing
   - Partial parses are valid and useful; perfect parses are not required
   - ~90% structural accuracy is the target, not 100%

2. **Observation, Not Transformation**
   - We extract structure; we don't modify it
   - No code generation, no refactoring primitives
   - Read-only by design

3. **Explicit Uncertainty**
   - `UnknownNode` and `UnparsedBlock` are first-class citizens
   - We never silently drop content we can't parse
   - LLMs and users can see where our "vision" is unclear

4. **Language-Agnostic Abstraction**
   - The IR is intentionally shallow – just enough structure for navigation
   - Deep semantic understanding is deferred to higher layers
   - This keeps the core stable as languages evolve

---
