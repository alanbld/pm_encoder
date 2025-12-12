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

## Token Optimization (v1.1+)

### Why Truncation?

When working with large projects, you may encounter:
- **Token limits** in LLMs (e.g., ChatGPT's context window)
- **Cost concerns** with token-based pricing
- **Performance issues** with very large contexts

Intelligent truncation solves these problems while keeping the most valuable code.

### Example 4: Smart Truncation for LLM Context

```bash
# Truncate files to 500 lines each, using language-aware smart mode
./pm_encoder.py . --truncate 500 --truncate-mode smart | pbcopy
```

This command:
1. Analyzes each file's language (Python, JS, etc.)
2. Preserves imports, class/function signatures, and entry points
3. Adds helpful summaries showing what was truncated
4. Reduces token usage by 60-80% typically

### Example 5: Truncation with Statistics

```bash
# See exactly how much you're saving
./pm_encoder.py . --truncate 300 --truncate-stats -o context.txt
```

Output:
```
======================================================================
TRUNCATION REPORT
======================================================================
Files analyzed: 45
Files truncated: 12 (26%)
Lines: 18,234 → 6,891 (62% reduction)

By Language:
  Python: 25 files, 8 truncated
  JavaScript/TypeScript: 15 files, 3 truncated
  Markdown: 5 files, 1 truncated

Estimated tokens: ~68,000 → ~26,000 (61% reduction)
======================================================================
```

### Example 6: Protect Important Files from Truncation

```bash
# Don't truncate README or LICENSE files
./pm_encoder.py . --truncate 400 \
  --truncate-exclude "README.md" "LICENSE" "CHANGELOG.md" \
  -o context.txt
```

### Understanding Smart vs Simple Truncation

**Simple Mode** (`--truncate-mode simple`):
- Just keeps the first N lines
- Fast, predictable
- Good for uniform files

**Smart Mode** (`--truncate-mode smart`):
- Analyzes file language and structure
- Preserves critical sections (imports, signatures, entry points)
- Adds detailed summaries
- Best for code understanding

Example smart truncation output:

```python
++++++++++ api/auth.py [TRUNCATED: 823 lines] ++++++++++
import jwt
from fastapi import HTTPException
from models import User

class AuthService:
    def __init__(self, secret_key: str):
        self.secret = secret_key

    def create_token(self, user_id: int) -> str:
        """Generate JWT token for user."""
        # ... (implementation details)

    def verify_token(self, token: str) -> dict:
        """Verify and decode JWT token."""
        # ... (implementation details)

... [650 lines omitted] ...

if __name__ == "__main__":
    # Development testing
    service = AuthService(os.getenv("SECRET_KEY"))
    print("Auth service initialized")

======================================================================
TRUNCATED at line 500/823 (39% reduction)
Language: Python
Category: Application Module
Classes (3): AuthService, TokenManager, SessionHandler
Functions (15): create_token, verify_token, refresh_token, ...
Key imports: jwt, fastapi, models, bcrypt, datetime

To get full content: --include "api/auth.py" --truncate 0
======================================================================
---------- api/auth.py [TRUNCATED:823→173] a1b2c3d4... ----------
```

### Example 7: Combining Filters and Truncation

```bash
# Complex workflow: Python files only, moderate truncation
./pm_encoder.py . \
  --include "src/**/*.py" "tests/**/*.py" \
  --truncate 600 \
  --truncate-mode smart \
  --truncate-exclude "tests/fixtures/*" \
  -o python_context.txt
```

### Workflow 4: Large Codebase to LLM

```bash
# 1. First, check the size without truncation
./pm_encoder.py . --include "*.py" "*.js" | wc -l
# Output: 45,000 lines (too big!)

# 2. Apply smart truncation
./pm_encoder.py . \
  --include "*.py" "*.js" \
  --truncate 400 \
  --truncate-mode smart \
  --truncate-stats \
  | tee context.txt | pbcopy

# 3. Review stats (stderr shows the report)
# Files truncated: 32/67 (47%)
# Estimated tokens: ~180K → ~52K (71% reduction)

# 4. Now paste into Claude/ChatGPT with your question
```

### Workflow 5: Iterative Context Refinement

```bash
# Start broad with heavy truncation
./pm_encoder.py . --truncate 200 --truncate-mode smart -o initial.txt

# Review truncation report, then get full version of specific files
./pm_encoder.py . \
  --include "src/critical_module.py" "src/another_key_file.py" \
  --truncate 0 \
  -o details.txt

# Combine both for hybrid context
```

### Tips for Token Optimization

**1. Right-size your truncation limit**
```bash
# Too aggressive (may lose important context)
./pm_encoder.py . --truncate 50

# Too conservative (may still hit limits)
./pm_encoder.py . --truncate 2000

# Sweet spot for most projects
./pm_encoder.py . --truncate 300-500
```

**2. Use smart mode for code, simple for data**
```bash
# Smart for source code
./pm_encoder.py src/ --truncate 500 --truncate-mode smart -o code.txt

# Simple for logs or data files
./pm_encoder.py logs/ --truncate 100 --truncate-mode simple -o logs.txt
```

**3. Exclude documentation from truncation**
```bash
# Keep full README and documentation
./pm_encoder.py . --truncate 400 --truncate-exclude "*.md" "docs/**"
```

**4. Preview before committing**
```bash
# Test truncation on a subset first
./pm_encoder.py src/critical_module/ --truncate 300 --truncate-mode smart
```

### Advanced: Language-Specific Truncation

Different languages are analyzed differently:

- **Python**: Preserves imports, class/function signatures, `__main__` blocks
- **JavaScript/TypeScript**: Preserves imports, exports, function/class declarations
- **Markdown**: Keeps all headers, first paragraph of each section
- **JSON/YAML**: Preserves structure, shows key names, samples values
- **Shell**: Keeps functions, sourced files, shebang

The smart mode automatically adapts to each language!

## Context Lenses (v1.2.0)

### What Are Context Lenses?

Context Lenses are pre-configured profiles that combine filters, sorting, and truncation strategies for specific use cases. Instead of manually setting multiple flags, you use one lens name.

### Example 8: Architecture Overview

Get a high-level view of your codebase structure:

```bash
# Shows only signatures - perfect for understanding APIs
./pm_encoder.py . --lens architecture | pbcopy
```

This automatically:
- Excludes tests, docs, and assets
- Uses structure mode (signatures only)
- Sorts by name for organized browsing
- Focuses on code structure, not implementation

**Perfect for:**
- Understanding a new codebase
- Sharing project structure with LLMs
- API documentation
- Code reviews focused on interfaces

### Example 9: Debug Session

Quickly grab recent changes for debugging:

```bash
# Shows full files, sorted by most recently modified
./pm_encoder.py . --lens debug -o recent.txt
```

This automatically:
- Shows full file content (no truncation)
- Sorts by modification time (newest first)
- Excludes only build artifacts
- Focuses on what changed recently

**Perfect for:**
- Bug investigation
- "What did I just break?"
- Recent feature development
- Code archaeology

### Example 10: Security Review

Focus on security-critical code:

```bash
# Smart truncation focused on security patterns
./pm_encoder.py . --lens security -o security_audit.txt
```

This automatically:
- Includes authentication, authorization, crypto code
- Excludes tests and documentation
- Smart truncation at 300 lines
- Preserves security-relevant patterns

**Perfect for:**
- Security audits
- Vulnerability scanning
- Compliance reviews
- Penetration testing prep

### Example 11: Team Onboarding

Create balanced overview for new developers:

```bash
# Moderate truncation with documentation
./pm_encoder.py . --lens onboarding -o welcome.txt
```

This automatically:
- Includes README, docs, and main code
- Smart truncation at 400 lines
- Preserves entry points and examples
- Balances breadth and depth

**Perfect for:**
- New team member onboarding
- Project handoffs
- Documentation generation
- High-level explanations

### Workflow 6: Custom Lens for Your Project

Create project-specific lenses in `.pm_encoder_config.json`:

```json
{
  "ignore_patterns": [".git", "node_modules"],
  "lenses": {
    "backend": {
      "description": "Backend API and database code",
      "include": ["api/**/*.py", "models/**/*.py", "db/**/*.sql"],
      "exclude": ["tests/**", "migrations/**"],
      "truncate_mode": "structure",
      "sort_by": "name"
    },
    "frontend": {
      "description": "React components and styles",
      "include": ["src/components/**", "src/pages/**", "src/styles/**"],
      "exclude": ["*.test.tsx", "*.stories.tsx"],
      "truncate_mode": "smart",
      "truncate": 300,
      "sort_by": "name"
    }
  }
}
```

Then use them:
```bash
# Backend context for API work
./pm_encoder.py . --lens backend -o backend.txt

# Frontend context for UI work
./pm_encoder.py . --lens frontend -o frontend.txt
```

### Understanding Lens Output

Every lens adds a `.pm_encoder_meta` file to explain the filtering:

```
++++++++++ .pm_encoder_meta ++++++++++
Context generated with lens: "architecture"
Focus: High-level structure, interfaces, configuration

Implementation details truncated using structure mode
Output shows only:
  - Import/export statements
  - Class and function signatures
  - Type definitions and interfaces
  - Module-level documentation

Generated: 2025-12-12T22:38:43.850133
pm_encoder version: 1.2.0
---------- .pm_encoder_meta ... ----------
```

This transparency helps LLMs (and humans) understand what they're seeing.

### Combining Lenses with CLI Flags

Lenses can be overridden with CLI flags:

```bash
# Use architecture lens, but override to smart mode instead of structure
./pm_encoder.py . --lens architecture --truncate-mode smart

# Use security lens, but also include tests
./pm_encoder.py . --lens security --include "tests/**"

# Use debug lens, but exclude a noisy file
./pm_encoder.py . --lens debug --exclude "logs/verbose.log"
```

**Precedence order:**
1. CLI flags (highest priority)
2. Lens settings
3. Config file
4. Defaults (lowest priority)

### Tips for Using Lenses

**1. Start with built-in lenses**
```bash
# Try each one to see what fits
./pm_encoder.py . --lens architecture | head -100
./pm_encoder.py . --lens debug | head -100
./pm_encoder.py . --lens security | head -100
```

**2. Create project-specific lenses**
```bash
# Add to .pm_encoder_config.json for your team
{
  "lenses": {
    "pr-review": {
      "description": "Code for pull request reviews",
      "truncate_mode": "smart",
      "truncate": 500,
      "sort_by": "mtime",
      "sort_order": "desc"
    }
  }
}
```

**3. Use structure mode for large codebases**
```bash
# Get overview without overwhelming LLMs
./pm_encoder.py . --lens architecture --truncate 100
```

**4. Combine with other tools**
```bash
# Architecture view of just changed files
git diff --name-only | xargs ./pm_encoder.py --lens architecture
```

## Next Steps

- Experiment with different include/exclude patterns for your project type
- Set up the configuration file to match your team's needs  
- Create shell aliases for your most common encoding patterns
- Consider using the backup script (`scripts/backup.sh`) to bundle your git repositories

For more details, see the main README.md file.