# Phase 1B: Core Fleet Expansion

## Objective
Extend `voyager-ast` to support Python and TypeScript, completing our "Core Fleet" of 3 high-priority languages:
1. Rust ✓ (Phase 1A)
2. Python
3. TypeScript

## Why These Languages?
- **Python**: Most popular for AI/ML, data science, scripting
- **TypeScript**: Dominant for web/frontend, growing in backend
- **Rust**: Systems programming, performance-critical tools

Together they cover ~70% of modern codebases.

## Technical Tasks

### 1. Python Adapter (`voyager-ast/src/adapters/python_adapter.rs`)
**Tree-sitter Grammar**: `tree-sitter-python`

**Key Constructs to Support:**
- Functions (`def`, `async def`)
- Classes and inheritance
- Imports (`import`, `from ... import`)
- Decorators
- Docstrings (triple-quoted strings)

**Special Considerations:**
- Significant whitespace (indentation-based blocks)
- Dynamic typing (type hints optional)
- List/dict comprehensions

### 2. TypeScript Adapter (`voyager-ast/src/adapters/typescript_adapter.rs`)
**Tree-sitter Grammar**: `tree-sitter-typescript`

**Key Constructs to Support:**
- Functions (named, arrow, async)
- Classes, interfaces, type aliases
- Imports/exports (ES6 modules)
- Generics (`<T>`)
- Decorators (@Component)

**Special Considerations:**
- Union/intersection types
- JSX/TSX (React components)
- Type vs value namespaces

### 3. Language Registry Enhancement
- Auto-detect `.py`, `.ts`, `.tsx`, `.js`, `.jsx`
- Fallback chain: AST → Heuristic → Unknown
- Priority routing to correct adapter

## Integration Requirements

### For VO Integration:
- Update `AstBridge` to recognize Python/TS files
- Maintain fallback to existing regex analyzers during transition
- Add integration tests for mixed-language projects

### Success Criteria:
1. **Functionality**: `vo .` works on Python/TS projects
2. **Accuracy**: >90% structural correctness on test corpus
3. **Performance**: Within Phase 1A latency targets
4. **Backwards Compatibility**: All existing tests still pass
5. **Jargon-Free**: Observatory metaphor preserved

## Testing Strategy

### Unit Tests (voyager-ast):
```rust
// Python
test_python_function_extraction()
test_python_class_inheritance()
test_python_import_statements()

// TypeScript
test_typescript_interface_extraction()
test_typescript_generic_functions()
test_typescript_jsx_elements()
```

### Integration Tests (VO):
```rust
test_mixed_python_rust_project()
test_typescript_react_project()
test_fallback_on_unsupported_language()
```

## Timeline Estimate
- **Week 1**: Python adapter + basic tests
- **Week 2**: TypeScript adapter + basic tests
- **Week 3**: Integration, performance tuning, edge cases
- **Week 4**: Documentation, final validation

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Tree-sitter grammar bugs | Test with real-world code samples |
| Performance regression | Benchmark early and often |
| Heuristic fallback failures | Maintain regex analyzers as backup |
| Memory usage increase | Stream parsing, avoid full AST in memory |

## Dependencies
- `tree-sitter-python` ^0.23.0
- `tree-sitter-typescript` ^0.23.0
- Update `voyager-ast/Cargo.toml` accordingly

## Success Metrics
- Python: 95%+ test coverage on standard library snippets
- TypeScript: 95%+ test coverage on Angular/React examples
- VO: Can analyze `vo ./python-project` with AST precision
- Performance: <100ms per file for Index mode

---

**Ready to begin when Phase 1A commit is complete.**
