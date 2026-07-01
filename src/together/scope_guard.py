from __future__ import annotations

from .file_policy import matches_any, normalize_path


def evaluate_scope_guard(contract: dict, changed_files: list[str], *, enforcement_mode: str | None = None) -> dict:
    scope_patterns = [normalize_path(item) for item in contract.get("scope", [])]
    denied_patterns = [normalize_path(item) for item in contract.get("denied_files", [])]
    mode = enforcement_mode or contract.get("enforcement_mode", "warn")

    inside_scope: list[str] = []
    out_of_scope_files: list[str] = []
    denied_files: list[str] = []

    for raw_path in changed_files:
        path = normalize_path(raw_path)
        if matches_any(path, denied_patterns):
            denied_files.append(path)
        elif not scope_patterns or matches_any(path, scope_patterns):
            inside_scope.append(path)
        else:
            out_of_scope_files.append(path)

    status = "PASS"
    if denied_files:
        status = "REJECT"
    elif out_of_scope_files:
        status = "REJECT" if mode == "strict" else "NEEDS_REVIEW"

    evidence = []
    if inside_scope:
        evidence.append(f"inside scope: {', '.join(inside_scope)}")
    if out_of_scope_files:
        evidence.append(f"out of scope: {', '.join(out_of_scope_files)}")
    if denied_files:
        evidence.append(f"denied files: {', '.join(denied_files)}")

    return {
        "status": status,
        "inside_scope_files": inside_scope,
        "out_of_scope_files": out_of_scope_files,
        "denied_files": denied_files,
        "evidence": evidence,
        "notes": [],
    }
