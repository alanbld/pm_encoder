#!/usr/bin/env python3
"""
Complexity Analysis for pm_encoder Research Phase 3

Measures code complexity metrics across Python and Rust implementations
to validate the hypothesis that Rust code has higher semantic density.

Metrics:
- NLOC: Non-Comment Lines of Code
- CCN: Cyclomatic Complexity Number (avg/max)
- Token Count: Halstead complexity proxy
- Semantic Density: Tokens / NLOC

Usage:
    python scripts/research/measure_complexity.py
"""

import json
import sys
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Any

try:
    import lizard
except ImportError:
    print("Error: lizard not installed. Run: uv pip install lizard")
    sys.exit(1)


@dataclass
class FileMetrics:
    """Metrics for a single file."""
    path: str
    nloc: int
    token_count: int
    function_count: int
    avg_ccn: float
    max_ccn: int
    max_ccn_function: str


@dataclass
class EngineMetrics:
    """Aggregated metrics for an engine (Python or Rust)."""
    engine: str
    total_nloc: int
    total_tokens: int
    total_functions: int
    avg_ccn: float
    max_ccn: int
    max_ccn_function: str
    semantic_density: float
    files: List[FileMetrics]


def analyze_file(filepath: str) -> FileMetrics:
    """Analyze a single file using lizard."""
    result = lizard.analyze_file(filepath)

    nloc = result.nloc
    token_count = result.token_count
    functions = result.function_list

    if functions:
        avg_ccn = sum(f.cyclomatic_complexity for f in functions) / len(functions)
        max_ccn_func = max(functions, key=lambda f: f.cyclomatic_complexity)
        max_ccn = max_ccn_func.cyclomatic_complexity
        max_ccn_function = max_ccn_func.name
    else:
        avg_ccn = 0.0
        max_ccn = 0
        max_ccn_function = "N/A"

    return FileMetrics(
        path=filepath,
        nloc=nloc,
        token_count=token_count,
        function_count=len(functions),
        avg_ccn=round(avg_ccn, 2),
        max_ccn=max_ccn,
        max_ccn_function=max_ccn_function
    )


def analyze_engine(engine_name: str, filepaths: List[str]) -> EngineMetrics:
    """Analyze all files for an engine and aggregate metrics."""
    file_metrics = []

    for filepath in filepaths:
        path = Path(filepath)
        if path.exists():
            metrics = analyze_file(str(path))
            file_metrics.append(metrics)
        else:
            print(f"Warning: File not found: {filepath}")

    if not file_metrics:
        return EngineMetrics(
            engine=engine_name,
            total_nloc=0,
            total_tokens=0,
            total_functions=0,
            avg_ccn=0.0,
            max_ccn=0,
            max_ccn_function="N/A",
            semantic_density=0.0,
            files=[]
        )

    total_nloc = sum(f.nloc for f in file_metrics)
    total_tokens = sum(f.token_count for f in file_metrics)
    total_functions = sum(f.function_count for f in file_metrics)

    # Calculate weighted average CCN
    all_ccns = []
    for fm in file_metrics:
        if fm.function_count > 0:
            all_ccns.extend([fm.avg_ccn] * fm.function_count)
    avg_ccn = sum(all_ccns) / len(all_ccns) if all_ccns else 0.0

    # Find max CCN across all files
    max_ccn_file = max(file_metrics, key=lambda f: f.max_ccn)

    # Calculate semantic density
    semantic_density = total_tokens / total_nloc if total_nloc > 0 else 0.0

    return EngineMetrics(
        engine=engine_name,
        total_nloc=total_nloc,
        total_tokens=total_tokens,
        total_functions=total_functions,
        avg_ccn=round(avg_ccn, 2),
        max_ccn=max_ccn_file.max_ccn,
        max_ccn_function=f"{max_ccn_file.path}::{max_ccn_file.max_ccn_function}",
        semantic_density=round(semantic_density, 2),
        files=file_metrics
    )


def print_comparison_table(python_metrics: EngineMetrics, rust_metrics: EngineMetrics):
    """Print a formatted comparison table."""
    print("\n" + "=" * 70)
    print("COMPLEXITY ANALYSIS: pm_encoder Research Phase 3")
    print("=" * 70)

    # Header
    print(f"\n{'Metric':<30} {'Python':<18} {'Rust':<18} {'Diff':<10}")
    print("-" * 70)

    # NLOC
    nloc_diff = rust_metrics.total_nloc - python_metrics.total_nloc
    nloc_pct = (nloc_diff / python_metrics.total_nloc * 100) if python_metrics.total_nloc else 0
    print(f"{'NLOC (Lines of Code)':<30} {python_metrics.total_nloc:<18} {rust_metrics.total_nloc:<18} {nloc_diff:+} ({nloc_pct:+.1f}%)")

    # Token Count
    token_diff = rust_metrics.total_tokens - python_metrics.total_tokens
    token_pct = (token_diff / python_metrics.total_tokens * 100) if python_metrics.total_tokens else 0
    print(f"{'Token Count':<30} {python_metrics.total_tokens:<18} {rust_metrics.total_tokens:<18} {token_diff:+} ({token_pct:+.1f}%)")

    # Function Count
    func_diff = rust_metrics.total_functions - python_metrics.total_functions
    print(f"{'Function Count':<30} {python_metrics.total_functions:<18} {rust_metrics.total_functions:<18} {func_diff:+}")

    # Average CCN
    ccn_diff = rust_metrics.avg_ccn - python_metrics.avg_ccn
    print(f"{'Avg Cyclomatic Complexity':<30} {python_metrics.avg_ccn:<18.2f} {rust_metrics.avg_ccn:<18.2f} {ccn_diff:+.2f}")

    # Max CCN
    max_diff = rust_metrics.max_ccn - python_metrics.max_ccn
    print(f"{'Max Cyclomatic Complexity':<30} {python_metrics.max_ccn:<18} {rust_metrics.max_ccn:<18} {max_diff:+}")

    # Semantic Density (key metric!)
    density_diff = rust_metrics.semantic_density - python_metrics.semantic_density
    density_pct = (density_diff / python_metrics.semantic_density * 100) if python_metrics.semantic_density else 0
    print(f"{'Semantic Density (Tok/NLOC)':<30} {python_metrics.semantic_density:<18.2f} {rust_metrics.semantic_density:<18.2f} {density_diff:+.2f} ({density_pct:+.1f}%)")

    print("-" * 70)

    # Max complexity functions
    print(f"\nHighest Complexity Functions:")
    print(f"  Python: {python_metrics.max_ccn_function} (CCN={python_metrics.max_ccn})")
    print(f"  Rust:   {rust_metrics.max_ccn_function} (CCN={rust_metrics.max_ccn})")

    # File breakdown
    print(f"\n{'File Breakdown':<30}")
    print("-" * 70)

    print("\nPython Files:")
    for f in python_metrics.files:
        print(f"  {f.path:<40} NLOC={f.nloc:<6} Tokens={f.token_count:<6} CCN={f.avg_ccn:.1f}")

    print("\nRust Files:")
    for f in rust_metrics.files:
        print(f"  {f.path:<40} NLOC={f.nloc:<6} Tokens={f.token_count:<6} CCN={f.avg_ccn:.1f}")

    # Hypothesis validation
    print("\n" + "=" * 70)
    print("HYPOTHESIS VALIDATION")
    print("=" * 70)

    if rust_metrics.semantic_density > python_metrics.semantic_density:
        print(f"\n[CONFIRMED] Rust has HIGHER semantic density ({rust_metrics.semantic_density:.2f} vs {python_metrics.semantic_density:.2f})")
        print(f"            {density_pct:.1f}% more meaning per line of code")
    else:
        print(f"\n[REJECTED] Rust has LOWER semantic density ({rust_metrics.semantic_density:.2f} vs {python_metrics.semantic_density:.2f})")

    if rust_metrics.avg_ccn > python_metrics.avg_ccn:
        print(f"\n[CONFIRMED] Rust has HIGHER average CCN ({rust_metrics.avg_ccn:.2f} vs {python_metrics.avg_ccn:.2f})")
        print(f"            Due to exhaustive pattern matching requirements")
    else:
        print(f"\n[NOTE] Rust has comparable/lower CCN ({rust_metrics.avg_ccn:.2f} vs {python_metrics.avg_ccn:.2f})")

    print("\n" + "=" * 70)


def save_results(python_metrics: EngineMetrics, rust_metrics: EngineMetrics, output_path: str):
    """Save results to JSON file."""
    results = {
        "metadata": {
            "tool": "lizard",
            "version": lizard.version,
            "description": "Complexity analysis for pm_encoder Research Phase 3"
        },
        "python": asdict(python_metrics),
        "rust": asdict(rust_metrics),
        "comparison": {
            "nloc_ratio": rust_metrics.total_nloc / python_metrics.total_nloc if python_metrics.total_nloc else 0,
            "token_ratio": rust_metrics.total_tokens / python_metrics.total_tokens if python_metrics.total_tokens else 0,
            "density_diff": rust_metrics.semantic_density - python_metrics.semantic_density,
            "ccn_diff": rust_metrics.avg_ccn - python_metrics.avg_ccn
        }
    }

    Path(output_path).parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to: {output_path}")


def main():
    """Main entry point."""
    # Define files to analyze
    python_files = [
        "pm_encoder.py"
    ]

    rust_files = [
        "rust/src/lib.rs",
        "rust/src/lenses.rs",
        "rust/src/analyzers/mod.rs",
        "rust/src/analyzers/generic.rs",
        "rust/src/analyzers/rust_analyzer.rs",
        "rust/src/bin/main.rs",
    ]

    # Check if we're in the right directory
    if not Path("pm_encoder.py").exists():
        print("Error: Run this script from the pm_encoder project root")
        sys.exit(1)

    print("Analyzing Python implementation...")
    python_metrics = analyze_engine("Python", python_files)

    print("Analyzing Rust implementation...")
    rust_metrics = analyze_engine("Rust", rust_files)

    # Print comparison table
    print_comparison_table(python_metrics, rust_metrics)

    # Save results
    save_results(python_metrics, rust_metrics, "research/data/complexity.json")


if __name__ == "__main__":
    main()
