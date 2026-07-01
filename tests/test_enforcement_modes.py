from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.file_policy import evaluate_file_policy
from together.quality_gate import evaluate_quality_gate
from together.scope_guard import evaluate_scope_guard
from together.verification_result import build_verification_result


def pass_verification(task_id: str = "TASK-123") -> dict:
    return build_verification_result(
        task_id=task_id,
        status="PASS",
        checks={
            "scope_compliance": {"status": "PASS", "evidence": [], "notes": []},
            "allowed_files": {"status": "PASS", "evidence": [], "notes": []},
            "denied_files": {"status": "PASS", "evidence": [], "notes": []},
            "acceptance_criteria": {"status": "PASS", "evidence": [], "notes": []},
            "routing_correctness": {"status": "PASS", "evidence": [], "notes": []},
            "architecture_compliance": {"status": "PASS", "evidence": [], "notes": []},
        },
    )


def test_warn_mode_marks_out_of_scope_as_needs_review() -> None:
    contract = {"task_id": "TASK-123", "scope": ["src/*.py"], "allowed_files": ["src/*.py"], "denied_files": [], "enforcement_mode": "warn"}
    result = evaluate_scope_guard(contract, ["README.md"])
    assert result["status"] == "NEEDS_REVIEW"


def test_strict_mode_rejects_out_of_scope_file() -> None:
    contract = {"task_id": "TASK-123", "scope": ["src/*.py"], "allowed_files": ["src/*.py"], "denied_files": [], "enforcement_mode": "strict"}
    result = evaluate_scope_guard(contract, ["README.md"])
    assert result["status"] == "REJECT"


def test_denied_file_always_rejects() -> None:
    contract = {"task_id": "TASK-123", "allowed_files": ["src/*.py"], "denied_files": ["src/secrets.py"], "enforcement_mode": "warn"}
    result = evaluate_file_policy(contract, ["src/secrets.py"])
    assert result["status"] == "REJECT"


def test_high_risk_requires_codex_approval() -> None:
    contract = {"task_id": "TASK-123", "risk_level": "high", "verification_required": True, "reviewer_required": True}
    gate = evaluate_quality_gate(contract, pass_verification(), review_status="PASS", task_status="passed", codex_approval=False)
    assert gate["status"] == "NEEDS_REVIEW"
    assert any("codex approval" in blocker.lower() for blocker in gate["blockers"])
