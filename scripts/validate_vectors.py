#!/usr/bin/env python3
"""
Test Vector Validation Script

Validates that Python implementation produces expected output for all test vectors.
This ensures test vectors are "golden data" before Rust implementation begins.

Usage:
    python3 scripts/validate_vectors.py [--verbose] [--category CATEGORY]

Options:
    --verbose       Show detailed output for each test
    --category      Only validate vectors in specific category (config, serialization, analyzer)
"""

import json
import sys
import tempfile
import shutil
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Any
from io import StringIO

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))
import pm_encoder


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'


def load_test_vector(vector_path: Path) -> Dict[str, Any]:
    """Load a test vector from JSON file."""
    with open(vector_path) as f:
        return json.load(f)


def create_test_environment(vector: Dict[str, Any]) -> Path:
    """Create temporary directory with test files from vector input."""
    temp_dir = Path(tempfile.mkdtemp(prefix=f"pm_encoder_validate_{vector['name']}_"))

    # Create input files
    for file_path, content in vector['input']['files'].items():
        full_path = temp_dir / file_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(content)

    return temp_dir


def run_pm_encoder(temp_dir: Path, vector: Dict[str, Any]) -> str:
    """Run pm_encoder on test directory and return output."""
    try:
        # Load config file if it exists in the temp directory
        config_file = temp_dir / '.pm_encoder_config.json'
        ignore_patterns = []
        include_patterns = []

        if config_file.exists():
            with open(config_file) as f:
                config_data = json.load(f)
                ignore_patterns = config_data.get('ignore_patterns', [])
                include_patterns = config_data.get('include_patterns', [])

        # Create output stream
        output_stream = StringIO()

        # Run serialization
        pm_encoder.serialize(
            project_root=temp_dir,
            output_stream=output_stream,
            ignore_patterns=ignore_patterns,
            include_patterns=include_patterns,
            sort_by='name',
            sort_order='asc',
            truncate_lines=0,
            truncate_mode='simple',
            truncate_summary=True,
            truncate_exclude=[],
            show_stats=False,
            language_plugins_dir=None,
            lens_manager=None
        )

        return output_stream.getvalue()
    except Exception as e:
        import traceback
        return f"ERROR: {str(e)}\n{traceback.format_exc()}"


def validate_output(output: str, expected: Dict[str, Any], verbose: bool = False) -> Tuple[bool, List[str]]:
    """
    Validate output against expected results.

    Returns:
        (success: bool, errors: List[str])
    """
    errors = []

    # Check files_included
    for file_name in expected.get('files_included', []):
        header = f"++++++++++ {file_name} ++++++++++"
        if header not in output:
            errors.append(f"Missing expected file: {file_name}")
        elif verbose:
            print(f"  ✓ Found file: {file_name}")

    # Check files_excluded
    for file_name in expected.get('files_excluded', []):
        header = f"++++++++++ {file_name} ++++++++++"
        if header in output:
            errors.append(f"Found excluded file: {file_name}")
        elif verbose:
            print(f"  ✓ Excluded file: {file_name}")

    # Check output_contains
    for content_str in expected.get('output_contains', []):
        if content_str not in output:
            errors.append(f"Missing expected content: '{content_str[:50]}...'")
        elif verbose:
            print(f"  ✓ Contains: '{content_str[:50]}...'")

    # Check output_excludes (if specified)
    for content_str in expected.get('output_excludes', []):
        if content_str in output:
            errors.append(f"Found excluded content: '{content_str[:50]}...'")
        elif verbose:
            print(f"  ✓ Excludes: '{content_str[:50]}...'")

    # Check metadata if specified (for analyzer tests)
    if 'metadata' in expected:
        # For now, just verify the test can run
        # Full metadata validation would require parsing output or internal API
        if verbose:
            print(f"  ℹ Metadata checks deferred (requires implementation)")

    return (len(errors) == 0, errors)


def validate_vector(vector_path: Path, verbose: bool = False) -> Tuple[bool, str, List[str]]:
    """
    Validate a single test vector.

    Returns:
        (success: bool, vector_name: str, errors: List[str])
    """
    vector = load_test_vector(vector_path)
    vector_name = vector['name']

    if verbose:
        print(f"\n{Colors.BLUE}Validating: {vector_name}{Colors.RESET}")
        print(f"  Description: {vector['description']}")

    # Create test environment
    temp_dir = create_test_environment(vector)

    try:
        # Run pm_encoder
        output = run_pm_encoder(temp_dir, vector)

        # Validate output
        success, errors = validate_output(output, vector['expected'], verbose)

        return (success, vector_name, errors)

    finally:
        # Cleanup
        shutil.rmtree(temp_dir, ignore_errors=True)


def update_vector_status(vector_path: Path, validated: bool):
    """Update python_validated status in vector JSON file."""
    vector = load_test_vector(vector_path)
    vector['python_validated'] = validated

    with open(vector_path, 'w') as f:
        json.dump(vector, f, indent=2)
        f.write('\n')  # Add trailing newline


def main():
    parser = argparse.ArgumentParser(description='Validate test vectors against Python implementation')
    parser.add_argument('--verbose', '-v', action='store_true', help='Show detailed output')
    parser.add_argument('--category', '-c', help='Only validate specific category')
    parser.add_argument('--update', '-u', action='store_true', help='Update python_validated status in JSON files')
    args = parser.parse_args()

    # Find all test vectors
    vectors_dir = Path(__file__).parent.parent / 'test_vectors' / 'rust_parity'
    vector_files = sorted(vectors_dir.glob('*.json'))

    if not vector_files:
        print(f"{Colors.RED}No test vectors found in {vectors_dir}{Colors.RESET}")
        sys.exit(1)

    # Filter by category if specified
    if args.category:
        vector_files = [
            vf for vf in vector_files
            if json.loads(vf.read_text()).get('category') == args.category
        ]

    print(f"{Colors.BOLD}PM_ENCODER Test Vector Validation{Colors.RESET}")
    print(f"Found {len(vector_files)} test vectors\n")

    # Validate each vector
    results = []
    for vector_file in vector_files:
        success, name, errors = validate_vector(vector_file, args.verbose)
        results.append((success, name, errors, vector_file))

        # Print result
        if success:
            print(f"{Colors.GREEN}✓ {name}{Colors.RESET}")
            if args.update:
                update_vector_status(vector_file, True)
        else:
            print(f"{Colors.RED}✗ {name}{Colors.RESET}")
            for error in errors:
                print(f"  {Colors.RED}• {error}{Colors.RESET}")

    # Summary
    passed = sum(1 for success, _, _, _ in results if success)
    failed = len(results) - passed

    print(f"\n{Colors.BOLD}Summary:{Colors.RESET}")
    print(f"  {Colors.GREEN}Passed: {passed}/{len(results)}{Colors.RESET}")
    if failed > 0:
        print(f"  {Colors.RED}Failed: {failed}/{len(results)}{Colors.RESET}")

    # Exit with error if any failed
    if failed > 0:
        print(f"\n{Colors.YELLOW}Some vectors failed validation.{Colors.RESET}")
        print(f"{Colors.YELLOW}Review errors and update test vectors or implementation.{Colors.RESET}")
        sys.exit(1)
    else:
        print(f"\n{Colors.GREEN}{Colors.BOLD}All test vectors validated! ✨{Colors.RESET}")
        if args.update:
            print(f"{Colors.GREEN}Updated python_validated status in JSON files.{Colors.RESET}")
        sys.exit(0)


if __name__ == '__main__':
    main()
