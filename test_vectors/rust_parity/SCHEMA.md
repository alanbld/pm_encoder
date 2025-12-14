# Rust Parity Test Vector Schema

Test vectors ensure Python and Rust engines produce identical output.

## Format

Each test vector is a JSON file with:

```json
{
  "name": "descriptive_test_name",
  "description": "What this test validates",
  "category": "config|serialization|analyzer|truncation|lens",
  "input": {
    "files": {
      "path/to/file": "content"
    },
    "config": {
      "ignore_patterns": [...],
      "include_patterns": [...]
    },
    "cli_args": ["--arg", "value"]
  },
  "expected": {
    "output_format": "plus_minus",
    "files_included": ["list", "of", "paths"],
    "files_excluded": ["list", "of", "paths"],
    "output_contains": ["strings", "that", "must", "appear"],
    "output_hash": "md5_of_entire_output",
    "metadata": {
      "file_count": 10,
      "total_lines": 1000
    }
  },
  "python_validated": true,
  "rust_status": "pending|passing|failing"
}
```

## Categories

1. **config**: Configuration loading and merging
2. **serialization**: Basic Plus/Minus format
3. **analyzer**: Language structure detection
4. **truncation**: Smart/structure truncation
5. **lens**: Lens system integration

## Workflow

1. Python generates test vector from existing test
2. Vector includes expected output (from Python run)
3. Rust test loads vector and validates against expected
4. 100% match = parity achieved

## Naming Convention

```
{category}_{sequence}_{description}.json

Examples:
config_01_file_loading.json
config_02_cli_override.json
serialization_01_basic_format.json
analyzer_01_python_class.json
```
