from __future__ import annotations

TASK_CONTRACT_FIELDS = (
    "task_id",
    "scope",
    "allowed_files",
    "denied_files",
    "deliverables",
    "success_criteria",
    "reviewer_required",
    "verification_required",
)

PERMISSION_ROLES = {
    "Observer": ["read", "search", "analyze"],
    "Researcher": ["research", "search", "summarize"],
    "Implementer": ["modify assigned scope only"],
    "Reviewer": ["review", "approve", "reject"],
    "Integrator": ["merge", "final decision"],
}

CODEX_ROLE = ("planner", "coordinator", "verifier", "integrator", "merge authority")

