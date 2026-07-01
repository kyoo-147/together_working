from __future__ import annotations

TASK_CONTRACT_FIELDS = (
    "task_id",
    "title",
    "department",
    "role",
    "owner",
    "scope",
    "allowed_files",
    "denied_files",
    "deliverables",
    "success_criteria",
    "reviewer_required",
    "verification_required",
    "merge_authority",
    "risk_level",
    "enforcement_mode",
    "unknown_files_policy",
)

PERMISSION_ROLES = {
    "Observer": ["read", "search", "analyze"],
    "Researcher": ["research", "search", "summarize"],
    "Implementer": ["modify assigned scope only"],
    "Reviewer": ["review", "approve", "reject"],
    "Integrator": ["merge", "final decision"],
}

CODEX_ROLE = ("planner", "coordinator", "verifier", "integrator", "merge authority")

RISK_LEVELS = ("low", "medium", "high")

ENFORCEMENT_MODES = ("warn", "strict")
