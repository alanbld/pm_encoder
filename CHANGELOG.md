# Changelog

All notable changes to pm_encoder will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-12-12

### Added - Intelligent Truncation System
- **Language-aware truncation**: Smart truncation that understands code structure across multiple languages
- **Built-in language analyzers** for:
  - Python (`.py`, `.pyw`): Classes, functions, imports, `__main__` blocks, docstrings, markers
  - JavaScript/TypeScript (`.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs`): Classes, functions, imports, exports, JSDoc
  - Shell (`.sh`, `.bash`, `.zsh`, `.fish`): Functions, sourced files, shebang detection
  - Markdown (`.md`, `.markdown`): Headers, code blocks, links, structure-aware truncation
  - JSON (`.json`): Structural analysis with key/depth detection
  - YAML (`.yaml`, `.yml`): Key structure preservation
- **Truncation modes**:
  - `simple`: Fast truncation keeping first N lines
  - `smart`: Language-aware truncation preserving critical code sections
- **Detailed truncation summaries** showing:
  - Language and file category (application/library/test/config)
  - Detected classes, functions, imports
  - Entry points and markers (TODO/FIXME)
  - Instructions for retrieving full content
- **Truncation statistics** with `--truncate-stats` flag showing:
  - Files analyzed vs truncated
  - Line and size reduction percentages
  - Per-language breakdown
  - Estimated token count reduction

### Added - Plugin System
- **Extensible language analyzer architecture** allowing community contributions
- **Plugin template generator**: `--create-plugin LANGUAGE` command
- **AI prompt generator**: `--plugin-prompt LANGUAGE` for getting AI assistance
- **Plugin loading system** from `~/.pm_encoder/plugins/` or custom directory
- **Example Rust analyzer** in `examples/plugins/rust_analyzer.py`
- **Comprehensive plugin development guide** (`PLUGIN_GUIDE.md`)

### Added - CLI Options
- `--truncate N`: Truncate files exceeding N lines (default: 0 = no truncation)
- `--truncate-mode {simple|smart}`: Choose truncation strategy (default: simple)
- `--truncate-summary`: Include analysis summary in truncation markers (default: true)
- `--no-truncate-summary`: Disable truncation summaries
- `--truncate-exclude PATTERN [PATTERN ...]`: Exclude files from truncation by glob pattern
- `--truncate-stats`: Show detailed truncation statistics report
- `--language-plugins DIR`: Specify custom language analyzer plugins directory
- `--create-plugin LANGUAGE`: Generate plugin template for a language
- `--plugin-prompt LANGUAGE`: Generate AI prompt for creating a plugin

### Changed
- **Enhanced Plus/Minus format**: Truncated files show `[TRUNCATED: N lines]` in headers and `[TRUNCATED:N→M]` in footers
- **Version bumped** to 1.1.0
- **Performance optimized**: Language analysis adds <100ms overhead per file

### Documentation
- **README.md**: Added language support matrix and truncation examples
- **TUTORIAL.md**: New "Token Optimization" section with practical truncation workflows
- **PLUGIN_GUIDE.md**: Complete guide for creating custom language analyzers
- **Examples**: Added `examples/plugins/` with Rust analyzer sample

### Technical Details
- Zero new external dependencies (still 100% standard library)
- Python 3.6+ compatibility maintained
- Backward compatible: existing workflows unchanged without truncation flags
- Regex-based analyzers (no AST parsing) for speed and portability

### Use Cases Unlocked
- **LLM context optimization**: Reduce large codebases to fit token limits
- **Cost reduction**: Lower API costs for token-based LLM services
- **Faster processing**: Smaller context = faster LLM responses
- **Better code understanding**: Summaries help AI grasp project structure
- **Multi-language projects**: Single tool handles polyglot repositories

## [1.0.0] - 2025-12-12

### Added
- Initial public release of pm_encoder
- Plus/Minus format serialization with MD5 checksums
- JSON configuration file support (`.pm_encoder_config.json`)
- CLI flags for filtering: `--include`, `--exclude`
- Sorting options: `--sort-by` (name, mtime, ctime) and `--sort-order` (asc, desc)
- Binary file detection (null-byte heuristic)
- Large file skipping (>5MB)
- UTF-8 encoding with latin-1 fallback
- POSIX-style paths in output for cross-platform compatibility
- Directory pruning for efficient traversal
- Standard output by default with `-o` option for file output

### Technical Details
- Python 3.6+ compatibility
- Zero external dependencies (standard library only)
- Single-file distribution (`pm_encoder.py`)

[1.0.0]: https://github.com/alanbld/pm_encoder/releases/tag/v1.0.0
