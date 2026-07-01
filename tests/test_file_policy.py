from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.file_policy import evaluate_file_policy


def test_file_policy_denied_wins() -> None:
    contract = {
        "task_id": "TASK-123",
        "allowed_files": ["src/together/*.py"],
        "denied_files": ["src/together/registry.py"],
    }
    result = evaluate_file_policy(contract, ["src/together/registry.py"], enforcement_mode="strict")
    assert result["status"] == "REJECT"
    assert result["denied_matched"] == ["src/together/registry.py"]


def test_file_policy_unknown_files_follow_policy() -> None:
    contract = {
        "task_id": "TASK-123",
        "allowed_files": ["src/together/routing.py"],
        "denied_files": [],
    }
    result = evaluate_file_policy(contract, ["README.md"], unknown_files_policy="needs_review")
    assert result["status"] == "NEEDS_REVIEW"
    assert result["unknown_matched"] == ["README.md"]
