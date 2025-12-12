# Changelog

All notable changes to pm_encoder will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
