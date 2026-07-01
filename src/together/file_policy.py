from __future__ import annotations

from fnmatch import fnmatchcase


def normalize_path(value: str) -> str:
    return value.replace("\\", "/").strip()


def matches_any(path: str, patterns: list[str]) -> bool:
    candidate = normalize_path(path)
    return any(fnmatchcase(candidate, normalize_path(pattern)) for pattern in patterns)


def evaluate_file_policy(
    contract: dict,
    changed_files: list[str],
    *,
    enforcement_mode: str | None = None,
    unknown_files_policy: str | None = None,
) -> dict:
    allowed_patterns = [normalize_path(item) for item in contract.get("allowed_files", [])]
    denied_patterns = [normalize_path(item) for item in contract.get("denied_files", [])]
    unknown_policy = unknown_files_policy or contract.get("unknown_files_policy", "needs_review")
    mode = enforcement_mode or contract.get("enforcement_mode", "warn")

    allowed_matched: list[str] = []
    denied_matched: list[str] = []
    unknown_matched: list[str] = []

    for raw_path in changed_files:
        path = normalize_path(raw_path)
        if matches_any(path, denied_patterns):
            denied_matched.append(path)
        elif not allowed_patterns or matches_any(path, allowed_patterns):
            allowed_matched.append(path)
        else:
            unknown_matched.append(path)

    status = "PASS"
    if denied_matched:
        status = "REJECT"
    elif unknown_matched:
        if unknown_policy == "allow":
            status = "PASS"
        elif unknown_policy == "reject" or mode == "strict":
            status = "REJECT"
        else:
            status = "NEEDS_REVIEW"

    evidence = {
        "allowed_matched": allowed_matched,
        "denied_matched": denied_matched,
        "unknown_matched": unknown_matched,
    }
    return {
        "status": status,
        **evidence,
        "evidence": [f"{name}: {', '.join(values)}" for name, values in evidence.items() if values],
        "notes": [],
    }
