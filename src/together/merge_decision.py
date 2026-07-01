from __future__ import annotations

from datetime import datetime, timezone


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def decide_merge(
    contract: dict,
    verification: dict | None,
    quality_gate: dict | None,
    task_state: dict | None,
    *,
    authority: str,
) -> dict:
    blockers: list[str] = []

    if authority != "codex":
        blockers.append("only codex may produce final merge decision")
    if contract.get("merge_authority", "codex") != "codex":
        blockers.append("contract merge_authority is invalid")
    if verification is None:
        blockers.append("missing verification evidence")
    if quality_gate is None:
        blockers.append("missing quality gate evidence")
    if task_state is None:
        blockers.append("missing task status evidence")

    verification_status = verification.get("status") if verification else None
    gate_status = quality_gate.get("status") if quality_gate else None
    task_status = task_state.get("status") if task_state else None

    if verification_status == "REJECT" or gate_status == "REJECT" or task_status == "rejected":
        blockers.append("task rejected by upstream evidence")
    elif verification_status != "PASS" or gate_status != "PASS" or task_status != "passed":
        blockers.append("task not ready for merge")

    decision = "MERGE"
    if any("only codex" in blocker or "missing" in blocker or "rejected" in blocker for blocker in blockers):
        decision = "REJECT"
    elif blockers:
        decision = "NEEDS_REVIEW"

    return {
        "task_id": contract.get("task_id"),
        "decision": decision,
        "authority": authority,
        "based_on": {
            "verification_status": verification_status,
            "quality_gate_status": gate_status,
            "task_status": task_status,
        },
        "reason": "; ".join(blockers) if blockers else "verification passed, quality gate passed, task passed",
        "timestamp": _now(),
    }
