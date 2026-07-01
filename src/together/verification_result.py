from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

VALID_VERIFICATION_STATUSES = {"PASS", "REJECT", "NEEDS_REVIEW"}
CHECK_NAMES = (
    "scope_compliance",
    "allowed_files",
    "denied_files",
    "acceptance_criteria",
    "routing_correctness",
    "architecture_compliance",
)


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def default_check(status: str = "NEEDS_REVIEW", evidence: list[str] | None = None, notes: list[str] | None = None) -> dict:
    return {
        "status": status,
        "evidence": list(evidence or []),
        "notes": list(notes or []),
    }


def build_verification_result(
    task_id: str,
    status: str = "NEEDS_REVIEW",
    checks: dict | None = None,
    verifier: str = "codex",
    department: str = "verification",
    violations: list[str] | None = None,
    notes: list[str] | None = None,
) -> dict:
    payload = {
        "task_id": task_id,
        "status": status,
        "verifier": verifier,
        "department": department,
        "checks": {name: default_check() for name in CHECK_NAMES},
        "violations": list(violations or []),
        "notes": list(notes or []),
        "timestamp": _now(),
    }
    if checks:
        payload["checks"].update(checks)
    return payload


def validate_verification_result(result: dict) -> list[str]:
    errors: list[str] = []
    if result.get("status") not in VALID_VERIFICATION_STATUSES:
        errors.append(f"invalid verification status: {result.get('status')}")
    if "task_id" not in result:
        errors.append("missing task_id")

    checks = result.get("checks", {})
    missing = set(CHECK_NAMES) - set(checks)
    if missing:
        errors.append(f"missing checks: {sorted(missing)}")

    for name in CHECK_NAMES:
        check = checks.get(name, {})
        if check.get("status") not in VALID_VERIFICATION_STATUSES:
            errors.append(f"{name}: invalid status")
        if not isinstance(check.get("evidence"), list):
            errors.append(f"{name}: evidence must be a list")
        if not isinstance(check.get("notes"), list):
            errors.append(f"{name}: notes must be a list")
    return errors


def load_verification_result(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))
