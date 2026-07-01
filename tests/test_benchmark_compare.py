from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "benchmark-compare.py"


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)


def test_benchmark_compare_renders_summary_report(tmp_path: Path) -> None:
    codex_only = tmp_path / "codex.json"
    together = tmp_path / "together.json"
    codex_only.write_text(
        json.dumps(
            {
                "task_id": "TASK-001",
                "mode": "codex_only",
                "metrics": {
                    "duration_seconds": 12.0,
                    "files_changed": 2,
                    "loc_changed": 10,
                    "tests": {"status": "PASS"},
                    "scope_violations": 1,
                    "quality_gate_result": None,
                    "merge_decision": None,
                    "human_rating": None,
                    "token_usage": {"status": "unavailable", "prompt_tokens": None, "completion_tokens": None, "total_tokens": None},
                },
            }
        ),
        encoding="utf-8",
    )
    together.write_text(
        json.dumps(
            {
                "task_id": "TASK-001",
                "mode": "together",
                "metrics": {
                    "duration_seconds": 14.5,
                    "files_changed": 2,
                    "loc_changed": 10,
                    "tests": {"status": "PASS"},
                    "scope_violations": 0,
                    "quality_gate_result": "PASS",
                    "merge_decision": "MERGE",
                    "human_rating": None,
                    "token_usage": {"status": "unavailable", "prompt_tokens": None, "completion_tokens": None, "total_tokens": None},
                },
            }
        ),
        encoding="utf-8",
    )

    proc = run([sys.executable, str(SCRIPT), str(codex_only), str(together)], tmp_path)

    assert proc.returncode == 0
    report = proc.stdout
    assert "# Benchmark Comparison" in report
    assert "TASK-001" in report
    assert "codex_only" in report
    assert "together" in report
    assert "scope_violations" in report
