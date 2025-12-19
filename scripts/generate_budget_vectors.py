#!/usr/bin/env python3
"""
Test Vector Generator for Token Budgeting (v1.7.0)

This script generates JSON test vectors that define expected behavior
for the Rust engine to validate against, following the Twins Protocol.

Vectors:
- budget_01_drop.json: Priority-based dropping
- budget_02_hybrid.json: Hybrid truncation strategy
- budget_03_lens_priority.json: Lens + budgeting integration

Usage:
    python3 scripts/generate_budget_vectors.py

Output:
    Creates/updates JSON files in test_vectors/
"""

import json
import sys
from pathlib import Path

# Add parent directory to path to import pm_encoder
sys.path.insert(0, str(Path(__file__).parent.parent))
import pm_encoder


def generate_budget_drop_vector():
    """
    Vector: Budget with Drop Strategy

    Tests:
    - Files are sorted by priority (highest first)
    - Low priority files are dropped when budget exceeded
    - High priority files are kept
    """
    print("Generating budget_01_drop.json...")

    # Create test files with varying sizes
    files = [
        ("high_priority.py", "class Main:\n    pass\n", 100),  # High priority
        ("medium_priority.py", "def helper():\n    pass\n", 50),  # Medium
        ("low_priority_test.py", "def test_something():\n    assert True\n", 10),  # Low (tests)
    ]

    # Simulate file content with tokens
    files_with_content = []
    for path, content, priority in files:
        files_with_content.append((Path(path), content))

    # Create a mock lens manager with groups
    class MockLensManager:
        def get_file_priority(self, path):
            path_str = str(path)
            if 'test' in path_str.lower():
                return 10  # Low priority for tests
            elif 'high' in path_str:
                return 100
            else:
                return 50

    lens_manager = MockLensManager()

    # Apply token budget with drop strategy
    budget = 100  # Small budget to force dropping
    selected, report = pm_encoder.apply_token_budget(
        files_with_content,
        budget,
        lens_manager,
        strategy="drop"
    )

    # Build vector
    vector = {
        "name": "budget_01_drop",
        "description": "Token budgeting with drop strategy - validates priority-based file selection",
        "version": "1.7.0",
        "category": "budgeting",
        "input": {
            "files": {path: content for path, content, _ in files},
            "budget": budget,
            "strategy": "drop",
            "priorities": {path: priority for path, _, priority in files}
        },
        "expected": {
            "strategy": "drop",
            "budget": budget,
            "files_selected": [str(p) for p, _ in selected],
            "files_dropped": [path for path, _, _ in report.dropped_files],
            "used_tokens": report.used,
            "dropped_count": report.dropped_count,
            "selected_count": report.selected_count,
            "behavior": [
                "Files sorted by priority DESC, then path ASC",
                "High priority files included first",
                "Low priority files dropped when budget exceeded"
            ]
        },
        "metadata": {
            "created_by": "generate_budget_vectors.py",
            "python_version": pm_encoder.__version__,
            "purpose": "Rust v0.8.0 budget drop strategy validation"
        }
    }

    return vector


def generate_budget_hybrid_vector():
    """
    Vector: Budget with Hybrid Strategy

    Tests:
    - Large files (>10% of budget) are auto-truncated
    - Truncated files still included with structure mode
    - Small files included as full
    """
    print("Generating budget_02_hybrid.json...")

    # Create files: one large Python file, one small
    large_python = """class LargeModule:
    '''A large module that exceeds 10% of budget.'''

    def __init__(self):
        self.data = []
        self.cache = {}
        self.config = {}

    def process(self, items):
        '''Process items with complex logic.'''
        result = []
        for item in items:
            processed = self._transform(item)
            result.append(processed)
        return result

    def _transform(self, item):
        '''Transform a single item.'''
        return item * 2

    def analyze(self, data):
        '''Analyze data and return statistics.'''
        total = sum(data)
        average = total / len(data) if data else 0
        return {'total': total, 'average': average}
"""

    small_python = "x = 1\n"

    files = [
        ("large_module.py", large_python * 5),  # Repeat to make it larger
        ("small.py", small_python),
    ]

    files_with_content = [(Path(p), c) for p, c in files]

    # Mock lens manager
    class MockLensManager:
        def get_file_priority(self, path):
            return 50  # Equal priority

    lens_manager = MockLensManager()

    # Get analyzer registry for truncation
    analyzer_registry = pm_encoder.LanguageAnalyzerRegistry()

    # Apply token budget with hybrid strategy
    budget = 500  # Budget where large file > 10%
    selected, report = pm_encoder.apply_token_budget(
        files_with_content,
        budget,
        lens_manager,
        strategy="hybrid",
        analyzer_registry=analyzer_registry
    )

    # Build vector
    vector = {
        "name": "budget_02_hybrid",
        "description": "Token budgeting with hybrid strategy - validates auto-truncation of large files",
        "version": "1.7.0",
        "category": "budgeting",
        "input": {
            "files": {p: c for p, c in files},
            "budget": budget,
            "strategy": "hybrid",
            "hybrid_threshold": 0.10
        },
        "expected": {
            "strategy": "hybrid",
            "budget": budget,
            "files_selected": [str(p) for p, _ in selected],
            "truncated_count": report.truncated_count,
            "selected_count": report.selected_count,
            "used_tokens": report.used,
            "behavior": [
                "Files > 10% of budget are auto-truncated to structure mode",
                "Truncated files preserve signatures, remove bodies",
                "Small files included in full"
            ]
        },
        "metadata": {
            "created_by": "generate_budget_vectors.py",
            "python_version": pm_encoder.__version__,
            "purpose": "Rust v0.8.0 budget hybrid strategy validation"
        }
    }

    return vector


def generate_budget_lens_priority_vector():
    """
    Vector: Budget with Lens Priority Groups

    Tests:
    - Lens priority groups affect file selection
    - Architecture lens prioritizes src/ over tests/
    - Integration of --lens and --token-budget
    """
    print("Generating budget_03_lens_priority.json...")

    # Create files that would be affected by architecture lens
    files = [
        ("src/core.py", "class Core:\n    pass\n"),
        ("src/utils.py", "def util():\n    pass\n"),
        ("tests/test_core.py", "def test_core():\n    assert True\n"),
        ("docs/readme.md", "# Documentation\n"),
    ]

    files_with_content = [(Path(p), c) for p, c in files]

    # Use the real lens manager with architecture lens
    lens_manager = pm_encoder.LensManager()
    # Set active lens manually for priority resolution
    lens_manager.active_lens = "architecture"

    # Apply token budget
    budget = 200
    selected, report = pm_encoder.apply_token_budget(
        files_with_content,
        budget,
        lens_manager,
        strategy="drop"
    )

    # Get priorities for each file
    priorities = {}
    for path, _ in files:
        priorities[path] = lens_manager.get_file_priority(Path(path))

    # Build vector
    vector = {
        "name": "budget_03_lens_priority",
        "description": "Token budgeting with lens priority groups - validates --lens + --token-budget integration",
        "version": "1.7.0",
        "category": "budgeting",
        "input": {
            "files": {p: c for p, c in files},
            "budget": budget,
            "strategy": "drop",
            "lens": "architecture"
        },
        "expected": {
            "strategy": "drop",
            "budget": budget,
            "priorities": priorities,
            "files_selected": [str(p) for p, _ in selected],
            "files_dropped": [path for path, _, _ in report.dropped_files],
            "selected_count": report.selected_count,
            "dropped_count": report.dropped_count,
            "behavior": [
                "Architecture lens assigns priorities via groups",
                "src/**/*.py gets higher priority than tests/**",
                "Files selected by priority order within budget"
            ]
        },
        "metadata": {
            "created_by": "generate_budget_vectors.py",
            "python_version": pm_encoder.__version__,
            "purpose": "Rust v0.8.0 lens + budget integration validation"
        }
    }

    return vector


def save_vector(vector, filename):
    """Save a test vector to test_vectors/ directory"""
    if vector is None:
        print(f"  [FAIL] Skipped {filename} (generation failed)")
        return False

    output_path = Path("test_vectors") / filename
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        json.dump(vector, f, indent=2)

    print(f"  [OK] Created {output_path}")
    return True


def main():
    """Generate all budget test vectors"""
    print("=" * 60)
    print("Test Vector Generator - Budget (v1.7.0 Intelligence Layer)")
    print("=" * 60)
    print()

    vectors = [
        (generate_budget_drop_vector, "budget_01_drop.json"),
        (generate_budget_hybrid_vector, "budget_02_hybrid.json"),
        (generate_budget_lens_priority_vector, "budget_03_lens_priority.json"),
    ]

    success_count = 0
    for generator, filename in vectors:
        try:
            vector = generator()
            if save_vector(vector, filename):
                success_count += 1
        except Exception as e:
            print(f"  [FAIL] {filename}: {e}")

    print()
    print("=" * 60)
    print(f"Generated {success_count}/{len(vectors)} test vectors")
    print("=" * 60)

    if success_count == len(vectors):
        print()
        print("[OK] All test vectors generated successfully!")
        print()
        print("Next steps:")
        print("  1. Review test_vectors/budget_*.json")
        print("  2. Run: cargo test --test test_vectors")
        print("  3. Fix any parity issues in Rust")
        return 0
    else:
        print()
        print("[WARN] Some test vectors failed to generate")
        return 1


if __name__ == "__main__":
    sys.exit(main())
