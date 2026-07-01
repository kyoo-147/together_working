from __future__ import annotations

import json
from pathlib import Path

from . import VERSION
from .registry import load_profiles

REPO_ROOT = Path(__file__).resolve().parents[2]
ROUTING_PATH = REPO_ROOT / "skills" / "together" / "data" / "capability-routing.json"

VALID_CAPABILITIES = {
    "vision",
    "backend",
    "frontend",
    "research",
    "review",
    "verification",
    "docs",
    "shell",
    "short_task",
    "long_task",
    "multi_file",
}

VALID_TASKS = VALID_CAPABILITIES | {"planning"}

VALID_DEPARTMENTS = {
    "planning",
    "research",
    "vision",
    "engineering",
    "review",
    "verification",
    "fallback",
}


def load_routing() -> dict:
    return json.loads(ROUTING_PATH.read_text(encoding="utf-8"))


def validate_routing_config(profile_doc: dict | None = None, routing_doc: dict | None = None) -> list[str]:
    profile_doc = profile_doc or load_profiles()
    routing_doc = routing_doc or load_routing()
    errors: list[str] = []
    if routing_doc.get("version") != VERSION:
        errors.append(f"routing version mismatch: expected {VERSION}, got {routing_doc.get('version')}")

    provider_ids = {provider["id"] for provider in profile_doc.get("providers", [])}
    task_routing = routing_doc.get("task_routing", {})
    departments = routing_doc.get("departments", {})

    for task, route in task_routing.items():
        if task not in VALID_TASKS:
            errors.append(f"unknown task capability in routing: {task}")
        for provider_id in route.get("preferred", []):
            if provider_id not in provider_ids:
                errors.append(f"task {task}: unknown provider {provider_id}")

    for department, preferred in departments.items():
        if department not in VALID_DEPARTMENTS:
            errors.append(f"unknown department: {department}")
        for provider_id in preferred:
            if provider_id not in provider_ids:
                errors.append(f"department {department}: unknown provider {provider_id}")

    for provider in profile_doc.get("providers", []):
        for hint in provider.get("capability_hints", []):
            if hint not in VALID_CAPABILITIES:
                errors.append(f"{provider['id']}: unknown capability hint {hint}")
        for department in provider.get("departments", []):
            if department not in VALID_DEPARTMENTS - {"fallback"}:
                errors.append(f"{provider['id']}: unknown department {department}")
    return errors
