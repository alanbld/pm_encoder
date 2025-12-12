# pm_encoder Plugin Development Guide

This guide explains how to create custom language analyzer plugins for pm_encoder to extend truncation support to additional programming languages.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Plugin Architecture](#plugin-architecture)
- [Creating a Plugin](#creating-a-plugin)
- [Testing Your Plugin](#testing-your-plugin)
- [Contributing Plugins](#contributing-plugins)
- [Examples](#examples)

## Overview

pm_encoder v1.1+ supports intelligent file truncation based on language-specific analysis. The plugin system allows you to add support for languages not included in the built-in analyzers.

### Built-in Language Support

pm_encoder includes analyzers for:

| Language | Extensions | Features Detected |
|----------|-----------|-------------------|
| Python | `.py`, `.pyw` | Classes, functions, imports, `__main__`, markers |
| JavaScript/TypeScript | `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs` | Classes, functions, imports, exports |
| Shell | `.sh`, `.bash`, `.zsh`, `.fish` | Functions, sourced files, shebang |
| Markdown | `.md`, `.markdown` | Headers, code blocks, links |
| JSON | `.json` | Keys, depth, structure |
| YAML | `.yaml`, `.yml` | Keys, structure |

## Quick Start

### 1. Generate a Plugin Template

```bash
./pm_encoder.py --create-plugin MyLanguage > my_language_analyzer.py
```

### 2. Customize the Template

Edit the generated file to add language-specific parsing logic:

```python
class LanguageAnalyzer:
    SUPPORTED_EXTENSIONS = ['.mylang', '.ml']  # Your file extensions
    LANGUAGE_NAME = "MyLanguage"

    def analyze(self, content: str, file_path: Path) -> Dict[str, Any]:
        # Add your language parsing here
        # Return structure with classes, functions, imports, etc.
        pass
```

### 3. Install the Plugin

```bash
mkdir -p ~/.pm_encoder/plugins/
cp my_language_analyzer.py ~/.pm_encoder/plugins/
```

### 4. Use Your Plugin

```bash
./pm_encoder.py . --truncate 500 --truncate-mode smart --language-plugins ~/.pm_encoder/plugins/
```

## Plugin Architecture

### The LanguageAnalyzer Interface

Every plugin must implement the `LanguageAnalyzer` class with two key methods:

```python
class LanguageAnalyzer:
    """Base interface for language analyzers."""

    SUPPORTED_EXTENSIONS = ['.ext']  # Required: List of file extensions
    LANGUAGE_NAME = "LanguageName"    # Required: Display name

    def analyze(self, content: str, file_path: Path) -> Dict[str, Any]:
        """
        Analyze file content and extract language constructs.

        Args:
            content: The complete file content as a string
            file_path: Path object for the file (may be None)

        Returns:
            Dictionary with keys:
            - language: str         # Language name
            - classes: List[str]    # Class/type names
            - functions: List[str]  # Function/method names
            - imports: List[str]    # Import statements
            - entry_points: List    # Main functions, exports
            - config_keys: List     # For config files
            - documentation: List   # Doc types found
            - markers: List         # TODO, FIXME, etc.
            - category: str         # application|library|test|config|documentation|script
            - critical_sections: List[Tuple[int, int]]  # (start_line, end_line) ranges
        """

    def get_truncate_ranges(self, content: str, max_lines: int) -> Tuple[List[Tuple[int, int]], Dict[str, Any]]:
        """
        Determine which line ranges to preserve during truncation.

        Args:
            content: The complete file content
            max_lines: Maximum number of lines to keep

        Returns:
            Tuple of (ranges, analysis) where:
            - ranges: List of (start_line, end_line) tuples (1-indexed)
            - analysis: Result from analyze() method
        """
```

### Analysis Return Structure

The `analyze()` method must return a dictionary with this structure:

```python
{
    "language": "Python",                    # Your language name
    "classes": ["MyClass", "Helper"],        # Classes/structs/types found
    "functions": ["main", "process", ...],   # Functions/methods found
    "imports": ["os", "sys", "re"],          # Import/require/use statements
    "entry_points": ["__main__"],            # Program entry points
    "config_keys": [],                       # Top-level keys (for config files)
    "documentation": ["docstrings"],         # Doc formats present
    "markers": ["TODO (line 42)", ...],      # Code markers found
    "category": "application",               # File categorization
    "critical_sections": [(150, 170), ...]   # Important line ranges
}
```

## Creating a Plugin

### Step-by-Step Example: Go Language Analyzer

#### 1. Generate Template

```bash
./pm_encoder.py --create-plugin Go > examples/plugins/go_analyzer.py
```

#### 2. Define Extensions and Patterns

```python
class LanguageAnalyzer:
    SUPPORTED_EXTENSIONS = ['.go']
    LANGUAGE_NAME = "Go"

    def analyze(self, content: str, file_path: Path) -> Dict[str, Any]:
        lines = content.split('\n')

        # Define regex patterns for Go constructs
        package_pattern = re.compile(r'^\s*package\s+(\w+)')
        struct_pattern = re.compile(r'^\s*type\s+(\w+)\s+struct')
        interface_pattern = re.compile(r'^\s*type\s+(\w+)\s+interface')
        func_pattern = re.compile(r'^\s*func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)')
        import_pattern = re.compile(r'^\s*"([^"]+)"')

        # ... continue implementation
```

#### 3. Parse Language Constructs

```python
        structs = []
        functions = []
        imports = []

        in_import_block = False

        for i, line in enumerate(lines, 1):
            # Handle package and imports
            if 'import (' in line:
                in_import_block = True
                continue
            if in_import_block and ')' in line:
                in_import_block = False
                continue

            if in_import_block or line.strip().startswith('import'):
                if match := import_pattern.search(line):
                    imports.append(match.group(1))

            # Parse structs
            if match := struct_pattern.match(line):
                structs.append(match.group(1))

            # Parse functions
            if match := func_pattern.match(line):
                fn_name = match.group(1)
                functions.append(fn_name)
                if fn_name == 'main':
                    entry_points.append(('func main', i))
```

#### 4. Categorize the File

```python
        # Determine file category
        category = "library"
        if 'main' in functions:
            category = "application"
        if file_path and '_test.go' in str(file_path):
            category = "test"

        return {
            "language": "Go",
            "classes": structs + interfaces,
            "functions": functions[:20],
            "imports": imports[:10],
            "entry_points": entry_points,
            "category": category,
            "critical_sections": [(ep[1], ep[1] + 15) for ep in entry_points]
        }
```

#### 5. Implement Smart Truncation Strategy

```python
    def get_truncate_ranges(self, content: str, max_lines: int) -> Tuple[List[Tuple[int, int]], Dict[str, Any]]:
        lines = content.split('\n')
        total_lines = len(lines)

        if total_lines <= max_lines:
            return [(1, total_lines)], self.analyze(content, None)

        analysis = self.analyze(content, None)

        # Go strategy: preserve package, imports, type definitions, main function
        keep_first = int(max_lines * 0.5)   # Package + imports + types
        keep_last = int(max_lines * 0.15)   # Exports and cleanup

        ranges = [(1, keep_first)]

        # Include main function if present
        if analysis["critical_sections"]:
            for start, end in analysis["critical_sections"]:
                if start > keep_first:
                    ranges.append((start - 3, min(end, total_lines)))

        # Add tail section
        if total_lines - keep_last > keep_first:
            ranges.append((total_lines - keep_last + 1, total_lines))

        return ranges, analysis
```

### Best Practices

1. **Use Regex, Not AST Parsing**: Keep plugins fast and dependency-free
2. **Be Conservative**: It's better to capture too much than too little
3. **Handle Edge Cases**: Empty files, malformed syntax, unusual formatting
4. **Limit List Sizes**: Truncate long lists (e.g., `functions[:20]`)
5. **Test on Real Code**: Use actual files from popular projects
6. **Performance**: Target <100ms per file analysis

### Common Patterns

#### Detecting Entry Points

```python
# Python
if '__name__' in line and '__main__' in line:
    entry_points.append(('__main__ block', i))

# JavaScript
if 'export default' in line:
    entry_points.append(('default export', i))

# Go/Rust/C
if 'func main' in line or 'fn main' in line or 'int main' in line:
    entry_points.append(('main function', i))
```

#### Handling Import Blocks

```python
# Multi-line imports (Go, Python from...import)
in_import_block = False
for line in lines:
    if line.startswith('import (') or line.startswith('from'):
        in_import_block = True
    if in_import_block and ')' in line:
        in_import_block = False
    if in_import_block:
        # Extract import
```

#### Finding Code Markers

```python
marker_pattern = re.compile(r'(?://|#)\s*(TODO|FIXME|XXX|HACK|NOTE):?\s*(.+)', re.IGNORECASE)
for i, line in enumerate(lines, 1):
    if match := marker_pattern.search(line):
        markers.append((match.group(1), match.group(2).strip(), i))
```

## Testing Your Plugin

### 1. Test on Sample Files

Create test files for your language:

```bash
mkdir -p test_files/
# Add sample files in your language
```

### 2. Run with Truncation

```bash
./pm_encoder.py test_files/ --truncate 50 --truncate-mode smart \
    --language-plugins ~/.pm_encoder/plugins/ -o output.txt
```

### 3. Verify Output

Check that the truncated output:
- Preserves imports/dependencies
- Keeps class/function signatures
- Includes entry points
- Shows useful summary in truncation markers

### 4. Test Edge Cases

- Empty files
- Very small files (<10 lines)
- Files with only comments
- Malformed syntax
- Unicode content

## Contributing Plugins

We welcome community contributions! To share your plugin:

### 1. Create a Pull Request

Submit your plugin to the `examples/plugins/` directory:

```
examples/plugins/
├── rust_analyzer.py
├── go_analyzer.py
├── kotlin_analyzer.py  ← Your plugin
└── ...
```

### 2. Include Documentation

Add a comment block at the top:

```python
"""
pm_encoder Language Plugin: Kotlin
Analyzer for Kotlin source files

Author: Your Name
Tested on: Kotlin 1.9+
Extensions: .kt, .kts

Example usage:
    ./pm_encoder.py . --truncate 500 --language-plugins ~/.pm_encoder/plugins/

Features detected:
    - Classes, data classes, objects
    - Functions and extension functions
    - Imports
    - Package declarations
    - Main functions
"""
```

### 3. Add Tests

Include a sample file showing your plugin in action:

```
examples/
├── plugins/
│   └── kotlin_analyzer.py
└── test_files/
    └── Sample.kt
```

### 4. Update Language Matrix

Add your language to the table in README.md:

```markdown
| Kotlin (community) | `.kt`, `.kts` | Classes, functions, imports |
```

## Examples

### Complete Plugin: Elixir

See `examples/plugins/elixir_analyzer.py` for a full example that detects:
- Modules (`defmodule`)
- Functions (`def`, `defp`)
- Imports (`import`, `alias`, `use`)
- Tests (`describe`, `test`)

### Using AI to Generate Plugins

Generate a prompt to get AI assistance:

```bash
./pm_encoder.py --plugin-prompt Elixir > elixir_prompt.txt
# Send elixir_prompt.txt to an AI assistant
```

The AI will receive a structured prompt with:
- Plugin interface specification
- Example template
- Requirements and constraints
- Example code structure

## Troubleshooting

### Plugin Not Loading

1. Check file location: `~/.pm_encoder/plugins/your_analyzer.py`
2. Verify class name is exactly `LanguageAnalyzer`
3. Ensure `SUPPORTED_EXTENSIONS` and `LANGUAGE_NAME` are defined

### Truncation Not Working

1. Verify your extension is in `SUPPORTED_EXTENSIONS`
2. Check that `get_truncate_ranges()` returns valid line numbers (1-indexed)
3. Test with `--truncate-mode smart` (not simple)

### Performance Issues

1. Avoid complex regex patterns
2. Limit iterations (don't parse every character)
3. Truncate large lists early
4. Use `re.compile()` outside loops

## Support

- **Issues**: https://github.com/alanbld/pm_encoder/issues
- **Discussions**: https://github.com/alanbld/pm_encoder/discussions
- **Examples**: See `examples/plugins/` directory

## License

All contributed plugins should be MIT licensed to match the pm_encoder project license.
