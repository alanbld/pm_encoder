"""
pm_encoder Language Plugin: Rust
Analyzer for Rust source files

Usage:
    1. Save to ~/.pm_encoder/plugins/rust_analyzer.py
    2. Use: ./pm_encoder.py . --truncate 500 --language-plugins ~/.pm_encoder/plugins/
"""

import re
from pathlib import Path
from typing import Dict, List, Tuple, Any


class LanguageAnalyzer:
    """Language analyzer for Rust."""

    SUPPORTED_EXTENSIONS = ['.rs']
    LANGUAGE_NAME = "Rust"

    def analyze(self, content: str, file_path: Path) -> Dict[str, Any]:
        """Analyze Rust file content and return structured information."""
        lines = content.split('\n')

        structs = []
        functions = []
        traits = []
        impls = []
        uses = []
        entry_points = []
        markers = []

        # Regex patterns for Rust
        struct_pattern = re.compile(r'^\s*(?:pub\s+)?struct\s+(\w+)')
        fn_pattern = re.compile(r'^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)')
        trait_pattern = re.compile(r'^\s*(?:pub\s+)?trait\s+(\w+)')
        impl_pattern = re.compile(r'^\s*impl(?:\s+<[^>]+>)?\s+(\w+)')
        use_pattern = re.compile(r'^\s*use\s+([^;]+);')
        marker_pattern = re.compile(r'//\s*(TODO|FIXME|XXX|HACK|NOTE):?\s*(.+)', re.IGNORECASE)

        for i, line in enumerate(lines, 1):
            # Structs
            if match := struct_pattern.match(line):
                structs.append(match.group(1))

            # Functions
            if match := fn_pattern.match(line):
                fn_name = match.group(1)
                functions.append(fn_name)
                if fn_name == 'main':
                    entry_points.append(('fn main', i))

            # Traits
            if match := trait_pattern.match(line):
                traits.append(match.group(1))

            # Impls
            if match := impl_pattern.match(line):
                impls.append(match.group(1))

            # Uses
            if match := use_pattern.match(line):
                uses.append(match.group(1).strip())

            # Markers
            if match := marker_pattern.search(line):
                markers.append((match.group(1), match.group(2).strip(), i))

        # Categorize
        category = "library"
        if 'main' in functions:
            category = "application"
        if file_path and ('test' in str(file_path).lower() or 'tests/' in str(file_path)):
            category = "test"

        return {
            "language": "Rust",
            "classes": structs + traits,
            "functions": functions[:20],
            "imports": uses[:10],
            "entry_points": [ep[0] for ep in entry_points],
            "config_keys": [],
            "documentation": ["doc comments"] if '///' in content or '//!' in content else [],
            "markers": [f"{m[0]} (line {m[2]})" for m in markers[:5]],
            "category": category,
            "critical_sections": [(ep[1], ep[1] + 20) for ep in entry_points]
        }

    def get_truncate_ranges(self, content: str, max_lines: int) -> Tuple[List[Tuple[int, int]], Dict[str, Any]]:
        """Determine which line ranges to keep when truncating."""
        lines = content.split('\n')
        total_lines = len(lines)

        if total_lines <= max_lines:
            return [(1, total_lines)], self.analyze(content, None)

        analysis = self.analyze(content, None)

        # Rust-specific strategy: preserve uses, struct/trait definitions, main function
        keep_first = int(max_lines * 0.5)  # For uses and type definitions
        keep_last = int(max_lines * 0.15)  # For module exports

        ranges = [(1, keep_first)]

        # Add main function if present
        if analysis["critical_sections"]:
            for start, end in analysis["critical_sections"]:
                if start > keep_first:
                    ranges.append((max(start - 5, keep_first + 1), min(end, total_lines)))

        # Add final section
        if total_lines - keep_last > keep_first:
            ranges.append((total_lines - keep_last + 1, total_lines))

        return ranges, analysis
