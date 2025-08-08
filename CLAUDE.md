# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **pm_encoder** project - a Python command-line utility that serializes project source files into a single text file using a custom "Plus/Minus" format. The tool is designed for sharing project context with LLMs, creating context packages for developers, or archival purposes.

## Key Architecture

- **Main Script**: `pm_encoder.py` - The core serialization utility (Python 3.6+)
- **Configuration**: `.pm_encoder_config.json` - JSON config file defining include/exclude patterns  
- **Backup Script**: `scripts/backup.sh` - Git repository backup utility using bundle format
- **Documentation**: `LLM/` directory contains enhancement specifications and development notes

## Common Commands

### Running the Encoder
```bash
# Make script executable (first time only)
chmod +x pm_encoder.py

# Basic usage - serialize current directory to stdout
./pm_encoder.py .

# Serialize to file using config
./pm_encoder.py . -o context.txt

# Pipe to clipboard (macOS)
./pm_encoder.py . | pbcopy

# Override include patterns
./pm_encoder.py . --include "*.py" "*.sh" -o scripts_only.txt

# Add exclude patterns temporarily
./pm_encoder.py . --exclude "*.log" "docs/" -o no_docs.txt

# Use custom config file
./pm_encoder.py . -c custom_config.json
```

### Backup Operations
```bash
# Run git bundle backup for the repository
./scripts/backup.sh
```

## Configuration System

The `.pm_encoder_config.json` file controls default filtering:
- `ignore_patterns`: Files/directories to exclude (supports glob patterns)
- `include_patterns`: Files to include (if empty, includes all non-ignored files)

Command-line flags (`--include`, `--exclude`) can override or extend config file settings.

## Plus/Minus Format

The tool outputs files in a structured format:
```
++++++++++ path/to/file.ext ++++++++++
[complete file content]
---------- path/to/file.ext <md5_checksum> path/to/file.ext ----------
```

## Key Features

- Binary file detection and automatic skipping
- Large file filtering (default: 5MB limit)
- Robust directory pruning for efficient traversal
- MD5 checksums for data integrity verification
- Support for UTF-8 and latin-1 text encodings
- Global file sorting capabilities (name, mtime, ctime)