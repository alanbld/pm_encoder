# Fractal Protocol v3.0: The Semantic Bridge Vision
Status: Vision / Research Phase
Target: "Universal Code Intelligence" via Hybrid Architecture

## 1. The Grand Vision: Beyond Skeletonization

We envision pm_encoder evolving into a **Universal Semantic Bridge** between local development environments and Large Language Models. Instead of merely providing a static snapshot of code, it will offer dynamic, context-aware intelligence about codebases in any language.

### 1.1 The Core Problem

LLMs are limited by the context window and lack of deep, semantic understanding of codebases. Current tools (including pm_encoder v2.2) rely on heuristics (regex, simple parsing) that are:

- **Fragile**: Break on complex syntax, new languages, or unusual patterns.
- **Shallow**: Cannot distinguish between public API and implementation details.
- **Manual**: Require per-language support and constant maintenance.

### 1.2 The Opportunity

The programming world has already solved the problem of semantic code understanding: **Language Server Protocol (LSP)**. Every modern IDE uses LSP for features like go-to-definition, find-references, and symbol outlining.

**Our Insight**: By integrating with LSP servers, pm_encoder can:

- Gain **perfect symbol extraction** for any language with an LSP server.
- Understand **code structure** (functions, classes, imports, references).
- Provide **intelligent code folding** (skeletonization) based on semantic importance.

### 1.3 The Ultimate Goal

Transform pm_encoder into a **Code Intelligence Service** that can answer queries about codebases, generate focused context for LLMs, and even create visualizations (like architecture diagrams) on demand.

---

## 2. The Quantum Leap: Capabilities Unlocked

### 2.1 Zero-Cost Language Support

Instead of writing and maintaining parsers for each language, we plug into existing LSP servers:

| Language | LSP Server | Notes |
|----------|------------|-------|
| Rust | rust-analyzer | Mature, high-quality |
| Python | pyright / jedi | Good type inference |
| Go | gopls | Official Go LSP |
| TypeScript/JavaScript | typescript-language-server | Node.js required |
| Java | eclipse.jdt.ls | Requires Java |
| C/C++ | clangd | Requires compilation database |
| Legacy/Enterprise | OpenEdge ABL LSP, COBOL LSP | Niche but critical for some |
| Documentation | markdown-lsp, asciidoctor-lsp | Structured text support |
| Configuration | terraform-lsp, ansible-lsp | Infrastructure as code |

**Implication**: We automatically support any language with an LSP server, including future languages.

### 2.2 Semantic Zoom 2.0

Move from simple text matching to semantic-aware navigation:

- **Current**: `zoom function=main` (finds text "main")
- **Future**:
  - `zoom reference=Config` (finds all usages of the Config type)
  - `zoom type=User` (shows the User struct/class and its methods)
  - `zoom diagram=class_hierarchy` (generates a class diagram via PlantUML)

### 2.3 Context-Aware Skeletonization

LSP provides metadata that allows intelligent compression:

- **Public vs. Private**: Keep public API signatures, fold private implementations.
- **Test vs. Production**: Fold test code when budget is tight.
- **Importance**: Prioritize recently modified or frequently referenced symbols.

### 2.4 Live Code Queries

Enable LLMs to ask dynamic questions about the codebase:

```bash
# Example: LLM can request specific context
pm_encode --query "find all API endpoints in the auth module"
pm_encode --query "show data flow from UserController to Database"
pm_encode --query "list all functions that call deprecated_method"
```

---

## 3. The Hybrid Architecture: Balancing Power and Practicality

We cannot force users to install 10 LSP servers just to run pm_encoder. We must maintain the simplicity and speed of a CLI tool.

### 3.1 The Three-Tiered Approach

| Tier | Engine | Speed | Accuracy | Dependencies | Use Case |
|------|--------|-------|----------|--------------|----------|
| Tier 1 | Native Regex | < 10ms | 90% (for supported langs) | None | Default, fast, CI |
| Tier 2 | Tree-Sitter | < 50ms | 99% (for supported langs) | Tree-sitter binaries | When available, for complex parsing |
| Tier 3 | LSP Bridge | 1-5s per language | 100% (for any LSP-supported lang) | External LSP servers | Deep analysis, niche languages |

### 3.2 Progressive Enhancement Algorithm

1. **Detect Languages** in the project (using file extensions, shebangs, etc.).
2. **Check Availability** of LSP servers for each language (via `$PATH` or configuration).
3. **Choose Engine** per language:
   - If LSP is available and the user has enabled it (or it's the default for that language), use Tier 3.
   - Else if Tree-sitter grammar is available, use Tier 2.
   - Else fall back to Tier 1 (regex) for known languages, or plain text for unknown.
4. **Parallelize** where possible (multiple LSP servers can be started simultaneously).

### 3.3 Caching and Performance

- **LSP Server Pool**: Keep LSP servers alive for the duration of the pm_encoder run to avoid startup costs per file.
- **Symbol Cache**: Cache extracted symbols per file (with modification time check) to avoid redundant LSP queries.
- **Timeout and Fallback**: If an LSP server takes too long (e.g., >2 seconds), fall back to a lower tier.

---

## 4. The Experiment: Validating Feasibility

Before committing to v3.0, we must answer critical questions:

### 4.1 Experiment Goals

1. **Latency**: How long does it take to start an LSP server, initialize it, and get symbols for a typical file?
   - Target: < 2 seconds total per language.

2. **Complexity**: How difficult is it to manage LSP lifecycle (stdio JSON-RPC) in Rust?
   - We must handle asynchronous communication, error handling, and cleanup.

3. **Value**: Does LSP provide significantly better symbol extraction than regex?
   - Compare accuracy on edge cases (nested functions, macros, decorators, etc.).

### 4.2 Proof of Concept Plan

Create a standalone binary `experiments/lsp_client` that:

1. Spawns a language server process (e.g., `rust-analyzer`).
2. Sends the `initialize` request with default capabilities.
3. Sends `textDocument/documentSymbol` for a sample file.
4. Measures the time for each step and prints the JSON response.
5. Extracts function/class signatures from the symbols and compares with regex output.

### 4.3 Metrics to Collect

- **Startup Time**: Time to spawn process and receive `initialized` notification.
- **First Symbol Time**: Time from `documentSymbol` request to response.
- **Accuracy**: Percentage of correctly extracted signatures (vs. hand-annotated ground truth).
- **Memory Usage**: RAM consumption of the LSP server.

### 4.4 Risk Assessment

- **Heavy Dependencies**: Users may not have LSP servers installed.
- **Slow Startup**: Some LSP servers are slow to initialize (e.g., Java-based servers).
- **Resource Consumption**: Multiple LSP servers could consume gigabytes of RAM.

---

## 5. Implementation Roadmap

### Phase 1: Research (Current)

- **Experiment**: Build `lsp_client` POC and collect metrics for 3-5 languages (Rust, Python, Go, TypeScript, Java).
- **Decision Point**: Based on results, decide whether to proceed with LSP integration or improve regex/tree-sitter.

### Phase 2: Core LSP Bridge (Optional Feature)

- **Design**: Define a clean abstraction for LSP-based symbol extraction.
- **Implement**: Support for 2-3 popular languages (Rust, Python, Go) as an optional feature (behind a feature flag).
- **Integration**: Allow pm_encoder to use LSP when available and enabled.

### Phase 3: Hybrid Skeletonization

- **Combine**: Use LSP symbols to guide skeletonization, fallback to regex for non-LSP files.
- **Optimize**: Implement caching and parallelization.

### Phase 4: Advanced Features

- **Semantic Zoom**: Implement reference finding and type-based zoom.
- **Query Interface**: Allow LLMs to ask structured queries about the codebase.

---

## 6. Challenges and Mitigations

### 6.1 Performance

- **Challenge**: LSP servers are slow to start and memory-hungry.
- **Mitigation**:
  - Use a persistent daemon (separate process) that keeps LSP servers warm.
  - Implement timeouts and fallbacks.
  - Allow users to disable LSP via configuration.

### 6.2 Dependencies

- **Challenge**: Users must install LSP servers.
- **Mitigation**:
  - Provide clear installation instructions.
  - Auto-detect and suggest installation (e.g., "No rust-analyzer found. Install it for better Rust support.").
  - Make LSP an opt-in feature.

### 6.3 Complexity

- **Challenge**: LSP protocol is complex (JSON-RPC, notifications, dynamic registration).
- **Mitigation**:
  - Use an existing LSP client library (e.g., `tower-lsp` for Rust) if possible.
  - Start with a minimal subset (only `documentSymbol` and `foldingRange`).

### 6.4 Determinism

- **Challenge**: LSP servers might give different results in different environments.
- **Mitigation**:
  - Cache results and provide a `--no-cache` flag for fresh analysis.
  - Use versioning in cache keys (so cache is invalidated when LSP server version changes).

---

## 7. Success Metrics

We will consider v3.0 a success if:

1. **Accuracy**: LSP-based skeletonization achieves >99% symbol extraction accuracy (on a test set of complex files).
2. **Performance**: The 95th percentile of processing time for a medium project (100 files) is under 5 seconds with LSP enabled (warm start).
3. **Adoption**: At least 30% of users enable LSP features for their primary language.

---

## 8. Conclusion

The LSP-based approach represents a **quantum leap** in code understanding for LLMs. It transforms pm_encoder from a simple text compression tool into a **universal semantic bridge**. However, the technical challenges are significant. We must proceed with careful experimentation and a phased rollout.

**Next Step**: Execute the `lsp_client` experiment and collect data. Let the data guide our decision.

---

*Document Version: 1.0*
*Last Updated: 2025-12-22*
*Author: pm_encoder Team*
