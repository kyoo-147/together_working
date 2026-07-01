from __future__ import annotations

import json
from pathlib import Path

from .state import TOGETHER_DIR

TASKS_DIR = TOGETHER_DIR / "tasks"

REQUIRED_CONTRACT_FIELDS = {
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
}

LIST_FIELDS = {"scope", "allowed_files", "denied_files", "deliverables", "success_criteria"}
BOOL_FIELDS = {"reviewer_required", "verification_required"}
OPTIONAL_DEFAULTS = {
    "risk_level": "medium",
    "enforcement_mode": "warn",
    "unknown_files_policy": "needs_review",
}
VALID_RISK_LEVELS = {"low", "medium", "high"}
VALID_ENFORCEMENT_MODES = {"warn", "strict"}
VALID_UNKNOWN_FILE_POLICIES = {"allow", "needs_review", "reject"}


def _parse_scalar(value: str):
    value = value.strip()
    if not value:
        return ""
    if value.startswith(("'", '"')) and value.endswith(("'", '"')) and len(value) >= 2:
        return value[1:-1]
    lowered = value.lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    return value


def _parse_simple_yaml(text: str) -> dict:
    data: dict = {}
    current_key: str | None = None

    for raw_line in text.splitlines():
        line = raw_line.rstrip()
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        if line.startswith((" ", "\t")):
            if current_key is None or not stripped.startswith("- "):
                raise ValueError(f"Unsupported YAML structure: {raw_line}")
            data.setdefault(current_key, []).append(_parse_scalar(stripped[2:]))
            continue

        current_key = None
        if ":" not in stripped:
            raise ValueError(f"Invalid YAML line: {raw_line}")
        key, raw_value = stripped.split(":", 1)
        key = key.strip()
        raw_value = raw_value.strip()
        if raw_value:
            data[key] = _parse_scalar(raw_value)
        else:
            data[key] = []
            current_key = key

    return data


def _normalize_contract(contract: dict) -> dict:
    normalized = dict(contract)
    for field, value in OPTIONAL_DEFAULTS.items():
        normalized.setdefault(field, value)
    for field in LIST_FIELDS:
        value = normalized.get(field, [])
        if isinstance(value, str):
            normalized[field] = [value]
        elif value is None:
            normalized[field] = []
        else:
            normalized[field] = list(value)
    return normalized


def load_contract(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    if path.suffix.lower() == ".json":
        contract = json.loads(text)
    else:
        contract = _parse_simple_yaml(text)
    return _normalize_contract(contract)


def load_optional_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def validate_contract(contract: dict) -> list[str]:
    errors: list[str] = []
    missing = REQUIRED_CONTRACT_FIELDS - set(contract)
    if missing:
        errors.append(f"missing fields: {sorted(missing)}")

    normalized = _normalize_contract(contract)

    for field in LIST_FIELDS:
        if not isinstance(normalized.get(field), list):
            errors.append(f"{field} must be a list")
    for field in BOOL_FIELDS:
        if not isinstance(normalized.get(field), bool):
            errors.append(f"{field} must be a boolean")
    if normalized.get("risk_level") not in VALID_RISK_LEVELS:
        errors.append(f"risk_level must be one of {sorted(VALID_RISK_LEVELS)}")
    if normalized.get("enforcement_mode") not in VALID_ENFORCEMENT_MODES:
        errors.append(f"enforcement_mode must be one of {sorted(VALID_ENFORCEMENT_MODES)}")
    if normalized.get("unknown_files_policy") not in VALID_UNKNOWN_FILE_POLICIES:
        errors.append(f"unknown_files_policy must be one of {sorted(VALID_UNKNOWN_FILE_POLICIES)}")
    if normalized.get("merge_authority") != "codex":
        errors.append("merge_authority must be codex")
    return errors


def contract_summary(contract: dict) -> dict:
    normalized = _normalize_contract(contract)
    return {
        "task_id": normalized["task_id"],
        "title": normalized["title"],
        "department": normalized["department"],
        "role": normalized["role"],
        "owner": normalized["owner"],
        "risk_level": normalized["risk_level"],
        "enforcement_mode": normalized["enforcement_mode"],
        "allowed_files": normalized["allowed_files"],
        "denied_files": normalized["denied_files"],
    }


def load_task_records(tasks_dir: Path = TASKS_DIR) -> tuple[list[dict], list[str]]:
    if not tasks_dir.exists():
        return [], []

    records: list[dict] = []
    warnings: list[str] = []

    for path in sorted(tasks_dir.glob("*.contract.yaml")):
        try:
            contract = load_contract(path)
        except Exception as exc:
            warnings.append(f"{path.name}: failed to load contract ({exc})")
            continue

        errors = validate_contract(contract)
        if errors:
            warnings.append(f"{path.name}: invalid contract ({'; '.join(errors)})")

        task_id = contract.get("task_id", path.stem.replace(".contract", ""))
        prefix = tasks_dir / task_id
        records.append(
            {
                "task_id": task_id,
                "contract_path": str(path),
                "contract": contract,
                "contract_errors": errors,
                "status": load_optional_json(prefix.with_suffix(".status.json")),
                "verification": load_optional_json(prefix.with_suffix(".verification.json")),
                "quality": load_optional_json(prefix.with_suffix(".quality.json")),
                "merge": load_optional_json(prefix.with_suffix(".merge.json")),
            }
        )

    return records, warnings
