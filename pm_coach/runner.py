"""
Differential Runner - Compare Python vs Rust pm_encoder output

This module executes both engines on the same repository and compares
their outputs, classifying any differences found.

Failure Types:
- OUTPUT_MISMATCH: Different serialized content
- CHECKSUM_MISMATCH: Same structure, different ordering
- MISSING_FILE: One engine includes a file the other doesn't
- EXTRA_FILE: One engine has more files
- ENCODING_DIFF: Unicode/encoding differences
- ANALYZER_DIFF: Different code analysis results
- CRASH: One engine crashed
- ARTIFACT_MISMATCH: Generated files differ (init-prompt mode)
"""

import difflib
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional, List, Dict, Any, Tuple

from .source import RepoSource


class FailureType(Enum):
    """Classification of differential test failures."""
    NONE = "none"                    # No failure (match)
    OUTPUT_MISMATCH = "output_mismatch"
    CHECKSUM_MISMATCH = "checksum_mismatch"
    MISSING_FILE = "missing_file"
    EXTRA_FILE = "extra_file"
    ENCODING_DIFF = "encoding_diff"
    ANALYZER_DIFF = "analyzer_diff"
    PYTHON_CRASH = "python_crash"
    RUST_CRASH = "rust_crash"
    BOTH_CRASH = "both_crash"
    ARTIFACT_MISMATCH = "artifact_mismatch"  # Generated files differ
    UNKNOWN = "unknown"


@dataclass
class RunResult:
    """Result of a differential test run."""
    repo_name: str
    repo_url: str
    match: bool
    failure_type: FailureType = FailureType.NONE

    # Outputs
    python_output: str = ""
    rust_output: str = ""
    python_stderr: str = ""
    rust_stderr: str = ""
    python_exit_code: int = 0
    rust_exit_code: int = 0

    # Analysis
    diff_summary: str = ""
    diff_lines: List[str] = field(default_factory=list)
    file_diff: Dict[str, str] = field(default_factory=dict)  # {path: "missing"|"extra"|"changed"}

    # Metadata
    python_time_ms: int = 0
    rust_time_ms: int = 0
    file_count: int = 0

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "repo_name": self.repo_name,
            "repo_url": self.repo_url,
            "match": self.match,
            "failure_type": self.failure_type.value,
            "diff_summary": self.diff_summary,
            "file_diff": self.file_diff,
            "python_exit_code": self.python_exit_code,
            "rust_exit_code": self.rust_exit_code,
            "python_time_ms": self.python_time_ms,
            "rust_time_ms": self.rust_time_ms,
            "file_count": self.file_count,
        }

    def save_vector(self, path: Path) -> None:
        """Save as test vector for reproduction."""
        vector = {
            "name": f"pm_coach_{self.repo_name}",
            "description": f"Auto-generated from {self.repo_url}",
            "category": "differential",
            "failure_type": self.failure_type.value,
            "repo_url": self.repo_url,
            "expected": {
                "match": True,
                "failure_type": "none",
            },
            "actual": {
                "match": self.match,
                "failure_type": self.failure_type.value,
                "diff_summary": self.diff_summary,
            },
            "reproduction": {
                "python_output_hash": hashlib.md5(self.python_output.encode()).hexdigest(),
                "rust_output_hash": hashlib.md5(self.rust_output.encode()).hexdigest(),
            }
        }
        with open(path, "w") as f:
            json.dump(vector, f, indent=2)


class DifferentialRunner:
    """Runs Python and Rust pm_encoder and compares output."""

    def __init__(
        self,
        python_cmd: str = "python3 pm_encoder.py",
        rust_cmd: str = "./rust/target/release/pm_encoder",
        lens: Optional[str] = None,
        verbose: bool = False,
    ):
        self.python_cmd = python_cmd
        self.rust_cmd = rust_cmd
        self.lens = lens
        self.verbose = verbose

        # Find project root (where pm_encoder.py lives)
        self.project_root = self._find_project_root()

    def _find_project_root(self) -> Path:
        """Find pm_encoder project root."""
        # Check common locations
        candidates = [
            Path.cwd(),
            Path.cwd().parent,
            Path(__file__).parent.parent,
        ]

        for path in candidates:
            if (path / "pm_encoder.py").exists():
                return path

        raise RuntimeError("Cannot find pm_encoder.py - run from project root")

    def _run_engine(self, cmd: str, repo_path: Path) -> tuple:
        """Run an encoder engine and capture output.

        Returns: (output, stderr, exit_code, time_ms)
        """
        import time

        # Build full command with absolute paths
        # Run with cwd=repo_path so both engines use target's config
        if cmd.startswith("python"):
            # Python: use absolute path to pm_encoder.py
            py_script = self.project_root / "pm_encoder.py"
            full_cmd = f"python3 {py_script} ."
        else:
            # Rust: use absolute path to binary
            rust_bin = self.project_root / "rust/target/release/pm_encoder"
            full_cmd = f"{rust_bin} ."

        if self.lens:
            full_cmd += f" --lens {self.lens}"

        start = time.time()
        try:
            result = subprocess.run(
                full_cmd,
                shell=True,
                capture_output=True,
                timeout=120,
                cwd=repo_path,  # Run from target dir for consistent config loading
            )
            elapsed_ms = int((time.time() - start) * 1000)

            return (
                result.stdout.decode("utf-8", errors="replace"),
                result.stderr.decode("utf-8", errors="replace"),
                result.returncode,
                elapsed_ms,
            )

        except subprocess.TimeoutExpired:
            return ("", "TIMEOUT", -1, 120000)
        except Exception as e:
            return ("", str(e), -2, 0)

    def compare(self, source: RepoSource) -> RunResult:
        """Run both engines and compare output."""
        meta = source.get_metadata()

        # We need the actual path for file-based engines
        # For ClonedRepoSource, access the internal path
        if hasattr(source, "_repo_path") and source._repo_path:
            repo_path = source._repo_path
        else:
            raise RuntimeError("RepoSource must expose filesystem path for comparison")

        # Run both engines
        py_out, py_err, py_code, py_time = self._run_engine(self.python_cmd, repo_path)
        rs_out, rs_err, rs_code, rs_time = self._run_engine(self.rust_cmd, repo_path)

        # Create result
        result = RunResult(
            repo_name=meta.name,
            repo_url=meta.url or "",
            match=False,
            python_output=py_out,
            rust_output=rs_out,
            python_stderr=py_err,
            rust_stderr=rs_err,
            python_exit_code=py_code,
            rust_exit_code=rs_code,
            python_time_ms=py_time,
            rust_time_ms=rs_time,
            file_count=meta.file_count,
        )

        # Classify failure
        result = self._classify(result)

        return result

    def _classify(self, result: RunResult) -> RunResult:
        """Classify the type of failure (if any)."""
        # Check for crashes
        if result.python_exit_code != 0 and result.rust_exit_code != 0:
            result.failure_type = FailureType.BOTH_CRASH
            result.diff_summary = f"Both crashed: Python={result.python_exit_code}, Rust={result.rust_exit_code}"
            return result

        if result.python_exit_code != 0:
            result.failure_type = FailureType.PYTHON_CRASH
            result.diff_summary = f"Python crashed: {result.python_stderr[:200]}"
            return result

        if result.rust_exit_code != 0:
            result.failure_type = FailureType.RUST_CRASH
            result.diff_summary = f"Rust crashed: {result.rust_stderr[:200]}"
            return result

        # Compare outputs
        if result.python_output == result.rust_output:
            result.match = True
            result.failure_type = FailureType.NONE
            result.diff_summary = "Identical output"
            return result

        # Outputs differ - classify
        py_lines = result.python_output.splitlines()
        rs_lines = result.rust_output.splitlines()

        # Check if same content different order (checksum mismatch)
        if sorted(py_lines) == sorted(rs_lines):
            result.failure_type = FailureType.CHECKSUM_MISMATCH
            result.diff_summary = "Same content, different order"
            return result

        # Compute diff
        differ = difflib.unified_diff(py_lines, rs_lines, lineterm="", n=0)
        diff_lines = list(differ)
        result.diff_lines = diff_lines[:50]  # Limit stored diff

        # Analyze diff for specific patterns
        added_lines = [l for l in diff_lines if l.startswith("+") and not l.startswith("+++")]
        removed_lines = [l for l in diff_lines if l.startswith("-") and not l.startswith("---")]

        # Check for file differences (look for +++ and --- patterns)
        py_files = self._extract_files(result.python_output)
        rs_files = self._extract_files(result.rust_output)

        missing = py_files - rs_files
        extra = rs_files - py_files

        if missing:
            result.failure_type = FailureType.MISSING_FILE
            result.diff_summary = f"Rust missing files: {missing}"
            result.file_diff = {f: "missing" for f in missing}
            return result

        if extra:
            result.failure_type = FailureType.EXTRA_FILE
            result.diff_summary = f"Rust extra files: {extra}"
            result.file_diff = {f: "extra" for f in extra}
            return result

        # Check for encoding patterns
        encoding_patterns = ["\\x", "\\u", "â", "ã", "ä"]
        if any(p in str(diff_lines) for p in encoding_patterns):
            result.failure_type = FailureType.ENCODING_DIFF
            result.diff_summary = f"Encoding difference ({len(diff_lines)} diff lines)"
            return result

        # Default to output mismatch
        result.failure_type = FailureType.OUTPUT_MISMATCH
        result.diff_summary = f"Output differs ({len(added_lines)} added, {len(removed_lines)} removed)"
        return result

    def _extract_files(self, output: str) -> set:
        """Extract file paths from pm_encoder output."""
        import re
        files = set()

        # Match the actual Plus/Minus header format:
        # ++++++++++ path/to/file.py ++++++++++
        pattern = r'^\+{10} ([^\s\[]+)'

        for line in output.splitlines():
            match = re.match(pattern, line)
            if match:
                path = match.group(1).strip()
                if path:
                    files.add(path)

        return files


@dataclass
class ArtifactResult:
    """Result of an artifact comparison (init-prompt mode)."""
    repo_name: str
    repo_url: str
    match: bool
    failure_type: FailureType = FailureType.NONE

    # Artifact files
    python_claude_md: str = ""
    rust_claude_md: str = ""
    python_context_txt: str = ""
    rust_context_txt: str = ""

    # Exit codes
    python_exit_code: int = 0
    rust_exit_code: int = 0

    # Diff info
    claude_md_diff: List[str] = field(default_factory=list)
    context_txt_diff: List[str] = field(default_factory=list)
    diff_summary: str = ""

    # Performance
    python_time_ms: int = 0
    rust_time_ms: int = 0

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "repo_name": self.repo_name,
            "repo_url": self.repo_url,
            "match": self.match,
            "failure_type": self.failure_type.value,
            "diff_summary": self.diff_summary,
            "python_exit_code": self.python_exit_code,
            "rust_exit_code": self.rust_exit_code,
            "python_time_ms": self.python_time_ms,
            "rust_time_ms": self.rust_time_ms,
            "claude_md_diff_lines": len(self.claude_md_diff),
            "context_txt_diff_lines": len(self.context_txt_diff),
        }


class ArtifactRunner:
    """Compares artifacts generated by --init-prompt (Split Brain mode).

    This runner tests the Split Brain architecture (v1.4.0+):
    - CLAUDE.md / GEMINI_INSTRUCTIONS.txt: Commands, tree, stats
    - CONTEXT.txt: Serialized codebase

    The standard: 0 bytes difference allowed.
    """

    def __init__(
        self,
        python_cmd: str = "python3 pm_encoder.py",
        rust_cmd: str = "./rust/target/release/pm_encoder",
        target: str = "claude",
        verbose: bool = False,
    ):
        self.python_cmd = python_cmd
        self.rust_cmd = rust_cmd
        self.target = target
        self.verbose = verbose

        # Find project root
        self.project_root = self._find_project_root()

        # Determine output filenames
        self.instruction_file = "CLAUDE.md" if target == "claude" else "GEMINI_INSTRUCTIONS.txt"

    def _find_project_root(self) -> Path:
        """Find pm_encoder project root."""
        candidates = [
            Path.cwd(),
            Path.cwd().parent,
            Path(__file__).parent.parent,
        ]

        for path in candidates:
            if (path / "pm_encoder.py").exists():
                return path

        raise RuntimeError("Cannot find pm_encoder.py - run from project root")

    def _run_init_prompt(self, cmd: str, repo_path: Path) -> Tuple[int, int, str]:
        """Run pm_encoder --init-prompt and capture results.

        Returns: (exit_code, time_ms, stderr)
        """
        import time

        # Build command with --init-prompt
        if cmd.startswith("python"):
            py_script = self.project_root / "pm_encoder.py"
            full_cmd = f"python3 {py_script} . --init-prompt --target {self.target}"
        else:
            rust_bin = self.project_root / "rust/target/release/pm_encoder"
            full_cmd = f"{rust_bin} . --init-prompt --target {self.target}"

        start = time.time()
        try:
            result = subprocess.run(
                full_cmd,
                shell=True,
                capture_output=True,
                timeout=120,
                cwd=repo_path,
            )
            elapsed_ms = int((time.time() - start) * 1000)
            return (
                result.returncode,
                elapsed_ms,
                result.stderr.decode("utf-8", errors="replace"),
            )
        except subprocess.TimeoutExpired:
            return (-1, 120000, "TIMEOUT")
        except Exception as e:
            return (-2, 0, str(e))

    def compare(self, source) -> ArtifactResult:
        """Compare artifacts generated by both engines.

        Workflow:
        1. Run Python --init-prompt
        2. Save CLAUDE.md and CONTEXT.txt as py_* versions
        3. Run Rust --init-prompt
        4. Save CLAUDE.md and CONTEXT.txt as rs_* versions
        5. Diff the files
        """
        meta = source.get_metadata()

        # Get repo path
        if hasattr(source, "_repo_path") and source._repo_path:
            repo_path = Path(source._repo_path)
        else:
            raise RuntimeError("RepoSource must expose filesystem path")

        result = ArtifactResult(
            repo_name=meta.name,
            repo_url=meta.url or "",
            match=False,
        )

        # Create temp directory for artifacts
        temp_dir = repo_path / "temp"
        temp_dir.mkdir(exist_ok=True)

        try:
            # === Step 1: Run Python --init-prompt ===
            py_code, py_time, py_err = self._run_init_prompt(self.python_cmd, repo_path)
            result.python_exit_code = py_code
            result.python_time_ms = py_time

            if py_code != 0:
                result.failure_type = FailureType.PYTHON_CRASH
                result.diff_summary = f"Python crashed: {py_err[:200]}"
                return result

            # Move Python artifacts
            py_claude = repo_path / self.instruction_file
            py_context = repo_path / "CONTEXT.txt"

            if py_claude.exists():
                result.python_claude_md = py_claude.read_text()
                shutil.move(str(py_claude), str(temp_dir / f"py_{self.instruction_file}"))

            if py_context.exists():
                result.python_context_txt = py_context.read_text()
                shutil.move(str(py_context), str(temp_dir / "py_CONTEXT.txt"))

            # === Step 2: Run Rust --init-prompt ===
            rs_code, rs_time, rs_err = self._run_init_prompt(self.rust_cmd, repo_path)
            result.rust_exit_code = rs_code
            result.rust_time_ms = rs_time

            if rs_code != 0:
                result.failure_type = FailureType.RUST_CRASH
                result.diff_summary = f"Rust crashed: {rs_err[:200]}"
                return result

            # Move Rust artifacts
            rs_claude = repo_path / self.instruction_file
            rs_context = repo_path / "CONTEXT.txt"

            if rs_claude.exists():
                result.rust_claude_md = rs_claude.read_text()
                shutil.move(str(rs_claude), str(temp_dir / f"rs_{self.instruction_file}"))

            if rs_context.exists():
                result.rust_context_txt = rs_context.read_text()
                shutil.move(str(rs_context), str(temp_dir / "rs_CONTEXT.txt"))

            # === Step 3: Compare artifacts ===
            result = self._compare_artifacts(result)

        finally:
            # Cleanup is handled by caller (ClonedRepoSource)
            pass

        return result

    def _compare_artifacts(self, result: ArtifactResult) -> ArtifactResult:
        """Compare the generated artifacts."""
        # Compare CLAUDE.md / GEMINI_INSTRUCTIONS.txt
        claude_match = result.python_claude_md == result.rust_claude_md
        context_match = result.python_context_txt == result.rust_context_txt

        if claude_match and context_match:
            result.match = True
            result.failure_type = FailureType.NONE
            result.diff_summary = "Identical artifacts (0 bytes difference)"
            return result

        # Generate diffs
        if not claude_match:
            py_lines = result.python_claude_md.splitlines()
            rs_lines = result.rust_claude_md.splitlines()
            diff = list(difflib.unified_diff(
                py_lines, rs_lines,
                fromfile=f"py_{self.instruction_file}",
                tofile=f"rs_{self.instruction_file}",
                lineterm="",
            ))
            result.claude_md_diff = diff[:100]  # Limit

        if not context_match:
            py_lines = result.python_context_txt.splitlines()
            rs_lines = result.rust_context_txt.splitlines()
            diff = list(difflib.unified_diff(
                py_lines, rs_lines,
                fromfile="py_CONTEXT.txt",
                tofile="rs_CONTEXT.txt",
                lineterm="",
            ))
            result.context_txt_diff = diff[:100]  # Limit

        result.failure_type = FailureType.ARTIFACT_MISMATCH
        result.diff_summary = (
            f"Artifacts differ: {self.instruction_file}={not claude_match}, "
            f"CONTEXT.txt={not context_match}"
        )

        if self.verbose:
            print(f"\n--- {self.instruction_file} diff ---")
            for line in result.claude_md_diff[:20]:
                print(line)
            print(f"\n--- CONTEXT.txt diff ---")
            for line in result.context_txt_diff[:20]:
                print(line)

        return result
