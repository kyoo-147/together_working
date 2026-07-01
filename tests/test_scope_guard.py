from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.scope_guard import evaluate_scope_guard


def test_scope_guard_passes_when_all_files_are_in_scope() -> None:
    contract = {
        "task_id": "TASK-123",
        "scope": ["src/together/*.py"],
        "allowed_files": ["src/together/routing.py"],
        "denied_files": [],
    }
    result = evaluate_scope_guard(contract, ["src/together/routing.py"])
    assert result["status"] == "PASS"
    assert result["out_of_scope_files"] == []


def test_scope_guard_rejects_denied_files() -> None:
    contract = {
        "task_id": "TASK-123",
        "scope": ["src/together/*.py"],
        "allowed_files": ["src/together/routing.py"],
        "denied_files": ["src/together/registry.py"],
    }
    result = evaluate_scope_guard(contract, ["src/together/routing.py", "src/together/registry.py"], enforcement_mode="strict")
    assert result["status"] == "REJECT"
    assert result["denied_files"] == ["src/together/registry.py"]
