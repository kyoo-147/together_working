from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.quality_gate import evaluate_quality_gate
from together.verification_result import build_verification_result


def test_quality_gate_passes_low_risk_with_verification() -> None:
    contract = {"task_id": "TASK-123", "risk_level": "low", "verification_required": True, "reviewer_required": False}
    verification = build_verification_result(
        task_id="TASK-123",
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
    gate = evaluate_quality_gate(contract, verification, review_status=None, task_status="passed")
    assert gate["status"] == "PASS"


def test_quality_gate_blocks_high_risk_without_codex_approval() -> None:
    contract = {"task_id": "TASK-123", "risk_level": "high", "verification_required": True, "reviewer_required": True}
    verification = build_verification_result(task_id="TASK-123", status="PASS")
    gate = evaluate_quality_gate(contract, verification, review_status="PASS", task_status="passed")
    assert gate["status"] == "NEEDS_REVIEW"
    assert "codex approval" in " ".join(gate["blockers"]).lower()
