from __future__ import annotations

from .verification_result import CHECK_NAMES, validate_verification_result

RISK_REQUIREMENTS = {
    "low": {"verification"},
    "medium": {"review", "verification"},
    "high": {"review", "verification", "codex_approval"},
}


def evaluate_quality_gate(
    contract: dict,
    verification: dict | None,
    *,
    review_status: str | None,
    task_status: str | None,
    codex_approval: bool = False,
) -> dict:
    risk_level = contract.get("risk_level", "medium")
    required_checks = sorted(RISK_REQUIREMENTS.get(risk_level, RISK_REQUIREMENTS["medium"]))
    blockers: list[str] = []
    check_results: dict[str, str] = {}

    if verification is None:
        blockers.append("missing verification result")
        check_results["verification"] = "MISSING"
    else:
        verification_errors = validate_verification_result(verification)
        if verification_errors:
            blockers.append("invalid verification result")
        verification_status = verification.get("status", "NEEDS_REVIEW")
        check_results["verification"] = verification_status
        if verification_status == "REJECT":
            blockers.append("verification rejected task")
        elif verification_status == "NEEDS_REVIEW":
            blockers.append("verification still needs review")

        missing_check_names = [name for name in CHECK_NAMES if name not in verification.get("checks", {})]
        if missing_check_names:
            blockers.append(f"missing required checks: {', '.join(missing_check_names)}")

        acceptance = verification.get("checks", {}).get("acceptance_criteria", {}).get("status")
        check_results["success_criteria"] = acceptance or "MISSING"
        if acceptance == "REJECT":
            blockers.append("acceptance criteria failed")
        elif acceptance != "PASS":
            blockers.append("acceptance criteria not verified")

    if "review" in required_checks:
        check_results["review"] = review_status or "MISSING"
        if review_status == "REJECT":
            blockers.append("review rejected task")
        elif review_status != "PASS":
            blockers.append("review required before integration")

    if "codex_approval" in required_checks:
        check_results["codex_approval"] = "PASS" if codex_approval else "MISSING"
        if not codex_approval:
            blockers.append("codex approval required for high-risk task")

    check_results["task_status"] = task_status or "MISSING"
    if task_status in {"rejected"}:
        blockers.append("task state is rejected")
    elif task_status in {"needs_review", "review_pending", "verification_pending", "paused", None}:
        blockers.append("task not ready for integration")

    reject_reasons = [message for message in blockers if "rejected" in message or "failed" in message or "missing verification" in message or "invalid verification" in message]
    status = "PASS"
    if reject_reasons:
        status = "REJECT"
    elif blockers:
        status = "NEEDS_REVIEW"

    return {
        "task_id": contract.get("task_id"),
        "status": status,
        "risk_level": risk_level,
        "required_checks": required_checks,
        "check_results": check_results,
        "blockers": blockers,
    }
