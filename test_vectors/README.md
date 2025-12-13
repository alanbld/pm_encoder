# Test Vectors - The Contract

Test vectors ensure Python and Rust engines produce identical output for identical input.

## Purpose

As pm_encoder evolves with **dual engines** (Python v1.3.1 and Rust v0.1.0), we need a mechanism to guarantee compatibility. Test vectors serve as the **contract** between implementations.

## Format

Each JSON file defines:
- **Input** (file content, configuration)
- **Expected Output** (structures, format, checksums)

Both engines must satisfy these specifications to maintain compatibility.

## Structure

```
test_vectors/
├── README.md                    # This file
├── basic_serialization.json     # Simple file serialization
├── python_analyzer.json         # Python structure extraction
└── *.json                       # Future test vectors
```

## Usage

### Python (Generator)

Python generates test vectors as the **reference implementation**:

```bash
python scripts/generate_test_vectors.py
```

### Rust (Validator)

Rust must reproduce expected output exactly (byte-identical):

```bash
cd rust && cargo test --test test_vectors
```

## Contract

1. **Python is the reference** - It defines expected behavior through test vectors
2. **Rust must match** - Rust implementation validates against test vectors
3. **Byte-identical output** - For the same input, both engines produce identical output
4. **Versioned compatibility** - Test vectors are versioned with engine releases

## Test Vector Schema

```json
{
  "name": "test_name",
  "description": "What this tests",
  "version": "1.0",
  "input": {
    "files": {
      "path/to/file": "content"
    },
    "config": {
      "truncate": true,
      "truncate_mode": "smart"
    }
  },
  "expected": {
    "format": "plus_minus",
    "contains": ["expected", "strings"],
    "structures": [
      {"type": "class", "name": "Foo", "line": 1}
    ],
    "checksum_present": true,
    "hash": "md5_of_output"
  }
}
```

## Adding New Test Vectors

1. Create a new JSON file in `test_vectors/`
2. Follow the schema above
3. Run Python generator to validate
4. Run Rust tests to ensure compatibility

## Why This Matters

**Without test vectors:**
- Python and Rust could drift apart
- Refactoring becomes risky
- Cross-engine compatibility breaks silently

**With test vectors:**
- ✅ Compatibility is enforced by tests
- ✅ Both engines stay synchronized
- ✅ Refactoring is safe (tests catch regressions)
- ✅ Future bindings (WASM, PyO3) inherit guarantees

## Cross-Engine Validation

Run `make test-cross` to compare outputs:

```bash
make test-cross
# Generates output from both engines and diffs them
```

## Status

**Current Coverage:**
- ✅ Basic file serialization
- ✅ Python structure extraction
- 🚧 Rust structure extraction (future)
- 🚧 Truncation modes (future)
- 🚧 Multi-file projects (future)

## Future Work

- [ ] Generate test vectors from real-world repos
- [ ] Fuzzing infrastructure
- [ ] Performance benchmarks
- [ ] Binary format test vectors (should be skipped)
- [ ] Unicode/encoding edge cases
