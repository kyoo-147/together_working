from __future__ import annotations

from datetime import datetime, timezone

VALID_TASK_STATES = {
    "planned",
    "assigned",
    "in_progress",
    "review_pending",
    "verification_pending",
    "passed",
    "rejected",
    "needs_review",
    "integrated",
    "paused",
}

ALLOWED_TRANSITIONS = {
    "planned": {"assigned", "paused"},
    "assigned": {"in_progress", "paused"},
    "in_progress": {"review_pending", "verification_pending", "needs_review", "rejected", "paused"},
    "review_pending": {"verification_pending", "needs_review", "rejected", "paused"},
    "verification_pending": {"passed", "needs_review", "rejected", "paused"},
    "passed": {"integrated", "paused"},
    "needs_review": {"assigned", "in_progress", "review_pending", "verification_pending", "paused"},
    "rejected": {"assigned", "paused"},
    "paused": {"assigned", "in_progress"},
    "integrated": set(),
}


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def create_task_state(task_id: str, department: str, owner: str, status: str = "planned") -> dict:
    if status not in VALID_TASK_STATES:
        raise ValueError(f"Unknown task state: {status}")
    timestamp = _now()
    return {
        "task_id": task_id,
        "status": status,
        "assigned_worker": None,
        "department": department,
        "owner": owner,
        "blocked_reason": None,
        "history": [
            {
                "status": status,
                "at": timestamp,
                "owner": owner,
            }
        ],
    }


def transition_task_state(
    state: dict,
    next_status: str,
    *,
    worker: str | None = None,
    owner: str | None = None,
    department: str | None = None,
    blocked_reason: str | None = None,
) -> dict:
    current = state["status"]
    if next_status not in VALID_TASK_STATES:
        raise ValueError(f"Unknown task state: {next_status}")
    if next_status not in ALLOWED_TRANSITIONS[current]:
        raise ValueError(f"Invalid transition: {current} -> {next_status}")

    updated = dict(state)
    updated["status"] = next_status
    updated["assigned_worker"] = worker if worker is not None else state.get("assigned_worker")
    updated["owner"] = owner if owner is not None else state.get("owner")
    updated["department"] = department if department is not None else state.get("department")
    updated["blocked_reason"] = blocked_reason

    history = list(state.get("history", []))
    history.append(
        {
            "status": next_status,
            "at": _now(),
            "owner": updated["owner"],
            "worker": updated["assigned_worker"],
            "blocked_reason": blocked_reason,
        }
    )
    updated["history"] = history
    return updated


def validate_task_state(state: dict) -> list[str]:
    errors: list[str] = []
    if state.get("status") not in VALID_TASK_STATES:
        errors.append(f"invalid task status: {state.get('status')}")
    if "history" not in state or not isinstance(state["history"], list) or not state["history"]:
        errors.append("history must be a non-empty list")
    return errors
