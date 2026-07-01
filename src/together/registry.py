from __future__ import annotations

import json
from pathlib import Path

from . import VERSION

REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = REPO_ROOT / "skills" / "together"
DATA_DIR = SKILL_ROOT / "data"
PROFILE_PATH = DATA_DIR / "agent-profiles.json"

REQUIRED_PROVIDER_FIELDS = {
    "id",
    "display_name",
    "commands",
    "lightweight_checks",
    "capability_hints",
    "departments",
    "confidence",
}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_profiles() -> dict:
    return load_json(PROFILE_PATH)


def validate_provider_profiles(profile_doc: dict) -> list[str]:
    errors: list[str] = []
    if profile_doc.get("version") != VERSION:
        errors.append(f"profile version mismatch: expected {VERSION}, got {profile_doc.get('version')}")

    seen_ids: set[str] = set()
    for provider in profile_doc.get("providers", []):
        missing = REQUIRED_PROVIDER_FIELDS - set(provider)
        if missing:
            errors.append(f"{provider.get('id', '<missing-id>')}: missing fields {sorted(missing)}")
        provider_id = provider.get("id")
        if provider_id in seen_ids:
            errors.append(f"duplicate provider id: {provider_id}")
        seen_ids.add(provider_id)
        if not provider.get("commands"):
            errors.append(f"{provider_id}: commands must not be empty")
        if not provider.get("capability_hints"):
            errors.append(f"{provider_id}: capability_hints must not be empty")
        if not provider.get("departments"):
            errors.append(f"{provider_id}: departments must not be empty")
    return errors


def sanitize_registry_snapshot(snapshot: dict) -> dict:
    scrubbed = json.loads(json.dumps(snapshot))
    scrubbed.pop("repo_root", None)
    scrubbed.pop("cache_path", None)
    scrubbed.pop("report_path", None)
    if "operations" in scrubbed:
        scrubbed["operations"]["override_path"] = ".together/providers.override.json"
        scrubbed["operations"]["runtime_state_path"] = ".together/cache/runtime-state.json"
        scrubbed["operations"]["last_known_good_path"] = ".together/cache/last-known-good.json"
    for provider in scrubbed.get("providers", []):
        if provider.get("path"):
            provider["path"] = "<sanitized>"
    return scrubbed

