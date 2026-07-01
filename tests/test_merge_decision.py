from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.merge_decision import decide_merge
from together.verification_result import build_verification_result


def test_merge_decision_requires_codex_authority() -> None:
    verification = build_verification_result(task_id="TASK-123", status="PASS")
    quality_gate = {"status": "PASS", "blockers": []}
    task_state = {"status": "passed"}
    decision = decide_merge(
        {"task_id": "TASK-123", "merge_authority": "codex"},
        verification,
        quality_gate,
        task_state,
        authority="claude",
    )
    assert decision["decision"] == "REJECT"


def test_merge_decision_allows_merge_with_evidence() -> None:
    verification = build_verification_result(task_id="TASK-123", status="PASS")
    quality_gate = {"status": "PASS", "blockers": []}
    task_state = {"status": "passed"}
    decision = decide_merge(
        {"task_id": "TASK-123", "merge_authority": "codex"},
        verification,
        quality_gate,
        task_state,
        authority="codex",
    )
    assert decision["decision"] == "MERGE"
