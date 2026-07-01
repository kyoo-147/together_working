from __future__ import annotations

import json
from pathlib import Path

from .contracts import load_contract, validate_contract
from .file_policy import evaluate_file_policy
from .git_tools import list_changed_files
from .merge_decision import decide_merge
from .quality_gate import evaluate_quality_gate
from .scope_guard import evaluate_scope_guard
from .task_state import create_task_state, transition_task_state
from .verification_result import build_verification_result

VALIDATION_ARTIFACT_SUFFIXES = {
    "status": ".status.json",
    "verification": ".verification.json",
    "quality": ".quality.json",
    "merge": ".merge.json",
}


def compute_verification_status(scope_status: str, policy_status: str) -> str:
    if "REJECT" in {scope_status, policy_status}:
        return "REJECT"
    if "NEEDS_REVIEW" in {scope_status, policy_status}:
        return "NEEDS_REVIEW"
    return "PASS"


def build_validation_verification(contract: dict, scope_guard: dict, file_policy: dict) -> dict:
    verification_status = compute_verification_status(scope_guard["status"], file_policy["status"])
    unknown_notes = []
    if file_policy["unknown_matched"]:
        unknown_notes.append("Unknown files require operator review.")

    return build_verification_result(
        task_id=contract["task_id"],
        status=verification_status,
        checks={
            "scope_compliance": {
                "status": scope_guard["status"],
                "evidence": scope_guard["evidence"],
                "notes": scope_guard["notes"],
            },
            "allowed_files": {
                "status": "PASS" if not file_policy["unknown_matched"] else file_policy["status"],
                "evidence": file_policy["evidence"],
                "notes": unknown_notes,
            },
            "denied_files": {
                "status": "REJECT" if file_policy["denied_matched"] else "PASS",
                "evidence": [f"denied_matched: {', '.join(file_policy['denied_matched'])}"] if file_policy["denied_matched"] else [],
                "notes": [],
            },
            "acceptance_criteria": {
                "status": verification_status,
                "evidence": list(contract.get("success_criteria", [])),
                "notes": ["Acceptance criteria not executed automatically; boundary validation only."]
                if verification_status != "PASS"
                else ["Boundary validation passed for declared success criteria."],
            },
            "routing_correctness": {
                "status": "PASS",
                "evidence": ["validate-task does not modify routing state"],
                "notes": [],
            },
            "architecture_compliance": {
                "status": verification_status,
                "evidence": [f"changed_files: {', '.join(scope_guard['inside_scope_files'] + scope_guard['out_of_scope_files'] + scope_guard['denied_files'])}"]
                if (scope_guard["inside_scope_files"] or scope_guard["out_of_scope_files"] or scope_guard["denied_files"])
                else [],
                "notes": ["Architecture review still operator-owned."] if verification_status != "PASS" else [],
            },
        },
        violations=scope_guard["out_of_scope_files"] + file_policy["denied_matched"],
        notes=[],
    )


def build_task_state_for_validation(contract: dict, validation_status: str) -> dict:
    state = create_task_state(contract["task_id"], contract["department"], contract["owner"])
    state = transition_task_state(state, "assigned", worker=contract["owner"])
    state = transition_task_state(state, "in_progress", worker=contract["owner"])
    state = transition_task_state(state, "verification_pending", worker=contract["owner"])
    if validation_status == "PASS":
        return transition_task_state(state, "passed", worker=contract["owner"])
    if validation_status == "REJECT":
        return transition_task_state(state, "rejected", worker=contract["owner"], blocked_reason="validation rejected task")
    return transition_task_state(state, "needs_review", worker=contract["owner"], blocked_reason="validation needs operator review")


def write_validation_artifacts(tasks_dir: Path, task_id: str, status: dict, verification: dict, quality: dict, merge: dict) -> list[str]:
    tasks_dir.mkdir(parents=True, exist_ok=True)
    written: list[str] = []
    artifacts = {
        "status": status,
        "verification": verification,
        "quality": quality,
        "merge": merge,
    }
    for name, payload in artifacts.items():
        path = tasks_dir / f"{task_id}{VALIDATION_ARTIFACT_SUFFIXES[name]}"
        path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        written.append(str(path))
    return written


def validate_task(
    contract_path: Path,
    *,
    cwd: Path,
    changed_files: list[str] | None = None,
    staged: bool = False,
    base: str | None = None,
    mode: str | None = None,
    write_artifacts: bool = False,
) -> tuple[dict, int]:
    contract = load_contract(contract_path)
    contract_errors = validate_contract(contract)
    if contract_errors:
        return {"error": "invalid-contract", "contract_errors": contract_errors, "contract_path": str(contract_path)}, 2

    effective_mode = mode or contract.get("enforcement_mode", "warn")

    changed_payload = None
    if changed_files is None:
        changed_payload, diff_error = list_changed_files(cwd, staged=staged, base=base)
        if diff_error:
            return diff_error, 2
        changed_files = changed_payload["changed_files"]
    else:
        changed_files = [item.replace("\\", "/").strip().lstrip("./") for item in changed_files if item.strip()]
        changed_payload = {
            "base": base or "HEAD",
            "mode": "provided",
            "git_root": str(cwd),
            "changed_files": changed_files,
        }

    scope_guard = evaluate_scope_guard(contract, changed_files, enforcement_mode=effective_mode)
    file_policy = evaluate_file_policy(
        contract,
        changed_files,
        enforcement_mode=effective_mode,
        unknown_files_policy=contract.get("unknown_files_policy", "needs_review"),
    )
    verification = build_validation_verification(contract, scope_guard, file_policy)
    task_state = build_task_state_for_validation(contract, verification["status"])
    quality_gate = evaluate_quality_gate(
        contract,
        verification,
        review_status="PASS" if not contract.get("reviewer_required") else None,
        task_status=task_state["status"],
        codex_approval=False,
        enforcement_mode=effective_mode,
    )
    merge_decision = decide_merge(contract, verification, quality_gate, task_state, authority="codex")

    git_root = Path(changed_payload["git_root"])
    written_artifacts: list[str] = []
    if write_artifacts:
        written_artifacts = write_validation_artifacts(git_root / ".together" / "tasks", contract["task_id"], task_state, verification, quality_gate, merge_decision)

    result = {
        "task_id": contract["task_id"],
        "contract_path": str(contract_path),
        "mode": effective_mode,
        "changed_files": changed_payload,
        "scope_guard": scope_guard,
        "file_policy": file_policy,
        "verification": verification,
        "task_status": task_state,
        "quality_gate": quality_gate,
        "merge_decision": merge_decision,
        "artifacts_written": written_artifacts,
    }

    exit_code = 0
    if effective_mode == "strict" and (
        scope_guard["status"] == "REJECT"
        or file_policy["status"] == "REJECT"
        or quality_gate["status"] == "REJECT"
        or merge_decision["decision"] == "REJECT"
    ):
        exit_code = 1
    return result, exit_code
