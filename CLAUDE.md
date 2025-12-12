# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **pm_encoder** project - a Python command-line utility that serializes project source files into a single text file using a custom "Plus/Minus" format. The tool is designed for sharing project context with LLMs, creating context packages for developers, or archival purposes.

## Key Architecture

- **Main Script**: `pm_encoder.py` - The core serialization utility (Python 3.6+, no external dependencies)
- **Configuration**: `.pm_encoder_config.json` - JSON config file defining include/exclude patterns
- **Backup Script**: `scripts/backup.sh` - Git repository backup utility using bundle format
- **LLM/**: Enhancement specifications (SIPs - Suggested Improvement Proposals) for development planning

## Common Commands

### Running the Encoder
```bash
# Basic usage - serialize current directory to stdout
./pm_encoder.py .

# Serialize to file using config
./pm_encoder.py . -o context.txt

# Pipe to clipboard
./pm_encoder.py . | pbcopy        # macOS
./pm_encoder.py . | xclip -selection clipboard  # Linux

# Override include patterns
./pm_encoder.py . --include "*.py" "*.sh" -o scripts_only.txt

# Add exclude patterns temporarily
./pm_encoder.py . --exclude "*.log" "docs/" -o no_docs.txt

# Sorting options (global sort across all files)
./pm_encoder.py . --sort-by mtime --sort-order desc  # newest first
./pm_encoder.py . --sort-by ctime --sort-order asc   # oldest created first

# Use custom config file
./pm_encoder.py . -c custom_config.json
```

### Backup Operations
```bash
# Run git bundle backup (copies to ~/backups and ~/icloud/Alan/backups/)
./scripts/backup.sh
```

## Configuration System

The `.pm_encoder_config.json` file controls default filtering:
- `ignore_patterns`: Files/directories to exclude (supports glob patterns)
- `include_patterns`: Files to include (if empty, includes all non-ignored files)

CLI flags behavior:
- `--include` **overrides** config include patterns
- `--exclude` **adds to** config ignore patterns

## Plus/Minus Format

The tool outputs files in a structured format:
```
++++++++++ path/to/file.ext ++++++++++
[complete file content]
---------- path/to/file.ext <md5_checksum> path/to/file.ext ----------
```

Key behaviors: auto-skips binary files (null-byte detection) and large files (>5MB), tries UTF-8 then falls back to latin-1 encoding.