#!/usr/bin/env python3
"""
pm_coach CLI - Differential Testing for pm_encoder

Usage:
    pm_coach https://github.com/user/repo    # Test single repo
    pm_coach repos.txt                        # Test repos from file
    pm_coach --top-python 100                 # Test top 100 Python repos

Examples:
    # Single repo
    pm_coach https://github.com/psf/requests

    # Multiple repos from file (one URL per line)
    pm_coach my_repos.txt --output results/

    # Generate test vectors from failures
    pm_coach https://github.com/user/repo --generate-vectors
"""

import argparse
import json
import sys
from pathlib import Path
from typing import List, Optional

from .runner import DifferentialRunner, RunResult
from .sources.cloned import ClonedRepoSource


__version__ = "0.1.0"


def parse_args(args: Optional[List[str]] = None) -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        prog="pm_coach",
        description="Differential testing: Python vs Rust pm_encoder",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "target",
        help="Repository URL, file with URLs, or special selector (--top-python N)",
    )

    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=Path("pm_coach_results"),
        help="Output directory for results (default: pm_coach_results/)",
    )

    parser.add_argument(
        "--generate-vectors",
        action="store_true",
        help="Generate test vectors from failures",
    )

    parser.add_argument(
        "--python-cmd",
        default="python3 pm_encoder.py",
        help="Command to run Python encoder",
    )

    parser.add_argument(
        "--rust-cmd",
        default="./rust/target/release/pm_encoder",
        help="Command to run Rust encoder",
    )

    parser.add_argument(
        "--lens",
        choices=["architecture", "debug", "security", "onboarding"],
        help="Apply lens to both engines",
    )

    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose output",
    )

    parser.add_argument(
        "--version",
        action="version",
        version=f"pm_coach {__version__}",
    )

    return parser.parse_args(args)


def load_repo_urls(target: str) -> List[str]:
    """Load repository URLs from target (URL or file)."""
    # Check if target is a file
    if Path(target).exists():
        with open(target) as f:
            urls = [line.strip() for line in f if line.strip() and not line.startswith("#")]
        return urls

    # Check if target looks like a URL
    if target.startswith(("http://", "https://", "git@")):
        return [target]

    raise ValueError(f"Invalid target: {target} (not a URL or file)")


def main(argv: Optional[List[str]] = None) -> int:
    """Main entry point."""
    args = parse_args(argv)

    # Create output directory
    args.output.mkdir(parents=True, exist_ok=True)

    try:
        urls = load_repo_urls(args.target)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    print(f"pm_coach v{__version__}")
    print(f"Testing {len(urls)} repository(ies)")
    print(f"Output: {args.output}/")
    print("-" * 60)

    # Initialize runner
    runner = DifferentialRunner(
        python_cmd=args.python_cmd,
        rust_cmd=args.rust_cmd,
        lens=args.lens,
        verbose=args.verbose,
    )

    # Results
    results: List[RunResult] = []
    passed = 0
    failed = 0

    for i, url in enumerate(urls, 1):
        print(f"\n[{i}/{len(urls)}] {url}")

        try:
            with ClonedRepoSource(url) as source:
                meta = source.get_metadata()
                print(f"  Cloned: {meta.file_count} files, {meta.clone_time_ms}ms")

                result = runner.compare(source)
                results.append(result)

                if result.match:
                    print(f"  Result: PASS (identical output)")
                    passed += 1
                else:
                    print(f"  Result: FAIL - {result.failure_type}")
                    print(f"  Details: {result.diff_summary}")
                    failed += 1

                    if args.generate_vectors:
                        vector_path = args.output / f"vector_{result.repo_name}.json"
                        result.save_vector(vector_path)
                        print(f"  Vector: {vector_path}")

        except Exception as e:
            print(f"  Error: {e}")
            failed += 1

    # Summary
    print("\n" + "=" * 60)
    print(f"SUMMARY: {passed} passed, {failed} failed, {len(urls)} total")
    print(f"Parity: {passed/len(urls)*100:.1f}%" if urls else "N/A")

    # Save full results
    results_file = args.output / "results.json"
    with open(results_file, "w") as f:
        json.dump([r.to_dict() for r in results], f, indent=2)
    print(f"Results saved: {results_file}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
