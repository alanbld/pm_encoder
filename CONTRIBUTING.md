# Contributing to pm_encoder

Thank you for your interest in contributing to pm_encoder! This document provides guidelines and information for contributors.

## Development Philosophy

pm_encoder is a **meta-tool for AI collaboration** — it exists to facilitate effective context sharing between developers and AI assistants. Please review `SYSTEM_INSTRUCTIONS.md` for the full development protocol.

### Core Principles

1. **Format-Preserving**: The Plus/Minus format is sacred. Changes must maintain backward compatibility.
2. **Zero Dependencies**: Standard library only. No external packages.
3. **Python 3.6+**: Maintain compatibility with Python 3.6 and above.
4. **Utility-Focused**: Every feature must solve a real context-sharing pain point.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/alanbld/pm_encoder.git
   cd pm_encoder
   ```
3. Create a branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Making Changes

### Before You Code

- Read `SYSTEM_INSTRUCTIONS.md` for development protocol
- Check existing issues to avoid duplicate work
- For significant changes, open an issue first to discuss

### Code Style

- Follow existing code conventions
- Use type hints for public functions
- Prefer `pathlib.Path` over `os.path`
- Handle errors gracefully (never crash on bad input)
- Keep it simple — avoid over-engineering

### Quality Checklist

Before submitting, verify:

- [ ] `./pm_encoder.py --version` outputs correct version
- [ ] All unit tests pass: `python3 -m unittest tests/test_pm_encoder.py`
- [ ] `./pm_encoder.py . -o /tmp/test.txt` succeeds (self-serialization)
- [ ] Plus/Minus format output is valid
- [ ] No external dependencies added
- [ ] Code works on Python 3.6+
- [ ] New features include unit tests (see Testing section)

### Testing

**Run the comprehensive test suite:**
```bash
# Run all tests
python3 -m unittest tests/test_pm_encoder.py

# Run with verbose output
python3 -m unittest tests/test_pm_encoder.py -v

# Run specific test
python3 -m unittest tests.test_pm_encoder.TestStructureMode.test_structure_mode_trigger
```

**Self-serialization test:**
```bash
# Basic functionality test
./pm_encoder.py . -o /tmp/test_output.txt
echo $?  # Should be 0

# Verify checksum integrity
head -1 /tmp/test_output.txt  # Should start with ++++++++++
```

**Test requirements for new features:**
- All new features **must include unit tests** in `tests/test_pm_encoder.py`
- Tests should use only standard library (unittest, tempfile, shutil)
- Verify tests pass before submitting PR
- Add test descriptions in docstrings

## Submitting Changes

1. Commit your changes with clear messages:
   ```bash
   git commit -m "feat: Add depth limiting with --depth flag"
   ```

2. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

3. Open a Pull Request with:
   - Clear description of changes
   - Reference to any related issues
   - Test results (unit tests + self-serialization)

## Commit Message Format

Use conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `refactor:` Code change that neither fixes a bug nor adds a feature
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

## Reporting Issues

When reporting bugs, include:
- pm_encoder version (`./pm_encoder.py --version`)
- Python version (`python3 --version`)
- Operating system
- Minimal reproduction steps
- Expected vs actual behavior

## Feature Requests

For new features, consider:
- Does this help developers share better context with AI?
- Can it be implemented without external dependencies?
- Does it maintain format backward compatibility?

## Code of Conduct

- Be respectful and constructive
- Focus on technical merit
- Welcome newcomers

## Questions?

Open an issue with the `question` label or check existing documentation:
- `README.md` — Usage guide
- `TUTORIAL.md` — Step-by-step examples
- `SYSTEM_INSTRUCTIONS.md` — Development protocol

Thank you for contributing!
