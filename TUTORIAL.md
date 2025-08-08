# PM Encoder Tutorial

This tutorial walks you through using `pm_encoder.py` to serialize your project files for sharing with LLMs or team members.

## Quick Start

### Step 1: Make the script executable
```bash
chmod +x pm_encoder.py
```

### Step 2: Run your first encoding
```bash
# Serialize current directory and copy to clipboard (macOS)
./pm_encoder.py . | pbcopy

# Or save to a file
./pm_encoder.py . -o my_project_context.txt
```

That's it! Your project files are now serialized in the Plus/Minus format.

## Understanding the Output Format

The tool creates output like this:
```
++++++++++ src/main.py ++++++++++
#!/usr/bin/env python3
print("Hello, world!")
---------- src/main.py a1b2c3d4e5f6... src/main.py ----------

++++++++++ README.md ++++++++++
# My Project
This is a sample project.
---------- README.md f6e5d4c3b2a1... README.md ----------
```

- Files are wrapped with clear start (`++++`) and end (`----`) markers
- End markers include MD5 checksums for integrity verification
- Binary files are automatically skipped

## Practical Examples

### Example 1: Python Project Context for ChatGPT
```bash
# Include only Python files and docs
./pm_encoder.py . --include "*.py" "*.md" "requirements.txt" | pbcopy
```

### Example 2: Debugging - Focus on Core Files
```bash
# Exclude tests and build artifacts
./pm_encoder.py . --exclude "tests/" "*.pyc" "__pycache__" -o debug_context.txt
```

### Example 3: Code Review Package
```bash
# Create focused context for a specific module
./pm_encoder.py . --include "src/auth/**" "tests/test_auth.py" -o auth_review.txt
```

## Configuration File Setup

Create `.pm_encoder_config.json` in your project root:

```json
{
  "ignore_patterns": [
    ".git", ".venv", "__pycache__", "*.pyc", "node_modules"
  ],
  "include_patterns": [
    "*.py", "*.js", "*.md", "*.json", "*.yaml", "*.txt"
  ]
}
```

Now you can simply run:
```bash
./pm_encoder.py .
```

## Tips & Best Practices

### 1. Start with Defaults
The tool has sensible defaults - try it without configuration first:
```bash
./pm_encoder.py . | head -20  # Preview first 20 lines
```

### 2. Use Previews
Check what files will be included before generating large outputs:
```bash
# Dry run - see which files match your patterns
find . -name "*.py" | grep -v __pycache__ | head -10
```

### 3. Size Management
Large projects can create huge outputs. Use filters:
```bash
# Focus on specific directories
./pm_encoder.py . --include "src/**" "docs/**"

# Or exclude heavy directories
./pm_encoder.py . --exclude "data/" "logs/" "*.log"
```

### 4. Clipboard Integration
Set up aliases for common workflows:
```bash
# Add to your .bashrc or .zshrc
alias encode-py="./pm_encoder.py . --include '*.py' '*.md' | pbcopy"
alias encode-all="./pm_encoder.py . | pbcopy"
```

## Common Workflows

### Workflow 1: LLM Context Generation
```bash
# 1. Navigate to your project
cd /path/to/my-project

# 2. Generate context with relevant files only
./pm_encoder.py . --include "*.py" "*.js" "*.md" "package.json" | pbcopy

# 3. Paste into ChatGPT/Claude with your question
```

### Workflow 2: Code Review Preparation
```bash
# 1. Create focused context for reviewers
./pm_encoder.py . --exclude "tests/" "docs/" "*.log" -o review_context.txt

# 2. Share the .txt file with your team
```

### Workflow 3: Bug Investigation
```bash
# 1. Include only files related to the problematic feature
./pm_encoder.py . --include "src/feature_x/**" "tests/test_feature_x.py" -o bug_context.txt

# 2. Attach to your bug report
```

## Troubleshooting

### "Permission denied" error
```bash
# Make sure script is executable
chmod +x pm_encoder.py
ls -la pm_encoder.py  # Should show -rwxr-xr-x
```

### Empty output
```bash
# Check if your include patterns are too restrictive
./pm_encoder.py . --include "**"  # Include everything (except ignored)
```

### Large output size
```bash
# Add more exclusions to your config
echo '{"ignore_patterns": [".git", "node_modules", "*.log", "data/"]}' > .pm_encoder_config.json
```

## Next Steps

- Experiment with different include/exclude patterns for your project type
- Set up the configuration file to match your team's needs  
- Create shell aliases for your most common encoding patterns
- Consider using the backup script (`scripts/backup.sh`) to bundle your git repositories

For more details, see the main README.md file.