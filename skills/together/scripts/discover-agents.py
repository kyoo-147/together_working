#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "src"))

from together.contracts import contract_summary, load_task_records

DATA_DIR = SKILL_ROOT / "data"
PROFILE_PATH = DATA_DIR / "agent-profiles.json"
ROUTING_PATH = DATA_DIR / "capability-routing.json"
TOGETHER_DIR = REPO_ROOT / ".together"
DEFAULT_CACHE = TOGETHER_DIR / "cache" / "agent-registry.json"
DEFAULT_LAST_KNOWN_GOOD = TOGETHER_DIR / "cache" / "last-known-good.json"
DEFAULT_RUNTIME_STATE = TOGETHER_DIR / "cache" / "runtime-state.json"
DEFAULT_OVERRIDE = TOGETHER_DIR / "providers.override.json"
DEFAULT_REPORT = TOGETHER_DIR / "reports" / "agent-report.md"
DEFAULT_TASKS_DIR = TOGETHER_DIR / "tasks"
DEFAULT_COOLDOWN_SECONDS = 900
VERSION = "0.5.0"

AUTH_FAIL_MARKERS = [
    "not configured",
    "not logged in",
    "not authenticated",
    "login required",
    "please login",
    "please log in",
    "authenticate first",
    "auth required",
]
AUTH_READY_MARKERS = [
    "authenticated as",
    "authentication verified",
    "logged in as",
]
PERMISSION_MARKERS = [
    "access is denied",
    "permission denied",
    "operation not permitted",
    "eacces",
]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_optional_json(path: Path, default: dict) -> dict:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def load_operator_json(path: Path, default: dict, label: str) -> tuple[dict, list[str]]:
    if not path.exists():
        return default, []
    try:
        return json.loads(path.read_text(encoding="utf-8-sig")), []
    except json.JSONDecodeError as exc:
        warning = (
            f"Ignored {label}: invalid JSON at line {exc.lineno}, column {exc.colno}. "
            "Using default config."
        )
        return default, [warning]
    except OSError as exc:
        warning = f"Ignored {label}: {exc}. Using default config."
        return default, [warning]


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def write_json(path: Path, data: dict) -> None:
    ensure_parent(path)
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")


def now_utc() -> datetime:
    return datetime.now(timezone.utc)


def parse_iso(value: str) -> datetime | None:
    try:
        return datetime.fromisoformat(value)
    except ValueError:
        return None


def run_check(command: str, found_path: str, raw_args: list[str]) -> tuple[bool, str]:
    shell = False
    run_target: list[str] | str = [command, *raw_args]

    if os.name == "nt" and found_path.lower().endswith((".cmd", ".bat")):
        run_target = subprocess.list2cmdline([found_path, *raw_args])
        shell = True

    try:
        proc = subprocess.run(
            run_target,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=12,
            shell=shell,
            check=False,
        )
    except Exception as exc:
        return False, str(exc)

    output = (proc.stdout or "") + ("\n" + proc.stderr if proc.stderr else "")
    return proc.returncode == 0, output.strip()


def classify_status(found_path: str | None, checks: dict[str, dict[str, str | bool]]) -> tuple[str, str]:
    if not found_path:
        return "not-installed", "No command found on PATH."

    outputs = "\n".join(str(result["output"]).lower() for result in checks.values())
    successes = [name for name, result in checks.items() if result["ok"]]
    failures = [name for name, result in checks.items() if not result["ok"]]

    if any(marker in outputs for marker in PERMISSION_MARKERS):
        return "permission-denied", "CLI exists but OS permissions blocked execution."
    if any(marker in outputs for marker in AUTH_FAIL_MARKERS):
        return "auth-required", "CLI exists but auth or config looks incomplete."
    if successes and any(marker in outputs for marker in AUTH_READY_MARKERS):
        return "ready", "CLI responded and auth markers look healthy."
    if successes:
        return "ready", "CLI responded to lightweight checks."
    if failures and len(failures) == len(checks):
        return "installed-but-broken", "CLI exists but all lightweight checks failed."
    return "installed-unknown", "CLI exists but readiness is unclear."


def unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return ordered


def apply_list_override(base: list[str], override: dict) -> list[str]:
    items = [item for item in base if item not in override.get("remove_preferred", [])]
    items = unique(override.get("preferred_first", []) + items + override.get("preferred_last", []))
    return items


def apply_overrides(profile_doc: dict, routing_doc: dict, override_doc: dict) -> tuple[dict, dict]:
    provider_overrides = override_doc.get("providers", {})
    for provider in profile_doc["providers"]:
        override = provider_overrides.get(provider["id"], {})
        provider["disabled"] = bool(override.get("disabled", False))
        provider["rank_adjust"] = int(override.get("rank_adjust", 0))
        disabled_capabilities = set(override.get("disable_capabilities", []))
        provider["capability_hints"] = [
            capability
            for capability in provider.get("capability_hints", [])
            if capability not in disabled_capabilities
        ]
        provider["capability_hints"] = unique(provider["capability_hints"] + override.get("add_capabilities", []))

    routing_override = override_doc.get("routing", {})
    for task, override in routing_override.get("tasks", {}).items():
        if task in routing_doc["task_routing"]:
            routing_doc["task_routing"][task]["preferred"] = apply_list_override(
                routing_doc["task_routing"][task]["preferred"], override
            )
    for department, override in routing_override.get("departments", {}).items():
        if department in routing_doc["departments"]:
            routing_doc["departments"][department] = apply_list_override(
                routing_doc["departments"][department], override
            )
    return profile_doc, routing_doc


def discover_provider(provider: dict) -> dict:
    if provider.get("disabled"):
        return {
            "id": provider["id"],
            "display_name": provider["display_name"],
            "commands": provider.get("commands", []),
            "command_found": None,
            "path": None,
            "installed": False,
            "health_ready": False,
            "routing_ready": False,
            "ready": False,
            "status": "disabled-by-override",
            "status_reason": "Provider disabled by local override.",
            "checks": {},
            "capability_hints": provider.get("capability_hints", []),
            "departments": provider.get("departments", []),
            "confidence": provider.get("confidence", "low"),
            "disabled": True,
            "rank_adjust": int(provider.get("rank_adjust", 0)),
        }

    found_command = None
    found_path = None
    for candidate in provider.get("commands", []):
        resolved = shutil.which(candidate)
        if resolved:
            found_command = candidate
            found_path = resolved
            break

    checks: dict[str, dict[str, str | bool]] = {}
    if found_command and found_path:
        for raw in provider.get("lightweight_checks", []):
            label = " ".join(raw)
            ok, output = run_check(found_command, found_path, raw)
            checks[label] = {"ok": ok, "output": output[:1200]}

    status, status_reason = classify_status(found_path, checks)
    installed = found_path is not None
    health_ready = status == "ready"

    return {
        "id": provider["id"],
        "display_name": provider["display_name"],
        "commands": provider.get("commands", []),
        "command_found": found_command,
        "path": found_path,
        "installed": installed,
        "health_ready": health_ready,
        "routing_ready": health_ready,
        "ready": health_ready,
        "status": status,
        "status_reason": status_reason,
        "checks": checks,
        "capability_hints": provider.get("capability_hints", []),
        "departments": provider.get("departments", []),
        "confidence": provider.get("confidence", "low"),
        "disabled": False,
        "rank_adjust": int(provider.get("rank_adjust", 0)),
    }


def compute_rank(provider: dict, status: str) -> int:
    base = {
        "ready": 100,
        "installed-unknown": 60,
        "auth-required": 40,
        "permission-denied": 30,
        "installed-but-broken": 20,
        "disabled-by-override": 0,
        "not-installed": 0,
    }.get(status, 0)
    confidence_bonus = {
        "high": 8,
        "medium": 4,
        "low": 1,
    }.get(provider.get("confidence", "low"), 0)
    return base + min(len(provider.get("capability_hints", [])), 8) + confidence_bonus + int(provider.get("rank_adjust", 0))


def load_runtime_state(path: Path, override_doc: dict) -> dict:
    state = load_optional_json(
        path,
        {
            "version": VERSION,
            "cooldown_seconds": DEFAULT_COOLDOWN_SECONDS,
            "recent_failures": {},
        },
    )
    state["cooldown_seconds"] = int(override_doc.get("runtime", {}).get("cooldown_seconds", state.get("cooldown_seconds", DEFAULT_COOLDOWN_SECONDS)))
    state["recent_failures"] = state.get("recent_failures", {})
    return state


def prune_runtime_state(runtime_state: dict, current_time: datetime) -> dict:
    kept: dict[str, dict] = {}
    for provider_id, record in runtime_state.get("recent_failures", {}).items():
        expires_at = parse_iso(str(record.get("cooldown_until", "")))
        if expires_at and expires_at > current_time:
            kept[provider_id] = record
    runtime_state["recent_failures"] = kept
    return runtime_state


def update_runtime_state(runtime_state: dict, providers: list[dict], current_time: datetime) -> dict:
    cooldown_seconds = int(runtime_state.get("cooldown_seconds", DEFAULT_COOLDOWN_SECONDS))
    recent_failures = dict(runtime_state.get("recent_failures", {}))
    cooldown_until = (current_time + timedelta(seconds=cooldown_seconds)).isoformat()

    for provider in providers:
        if provider["status"] in {"disabled-by-override", "not-installed"}:
            recent_failures.pop(provider["id"], None)
            continue
        if provider["health_ready"]:
            continue
        if not provider["installed"]:
            continue
        recent_failures[provider["id"]] = {
            "reason": provider["status"],
            "last_failed_at": current_time.isoformat(),
            "cooldown_until": cooldown_until,
        }

    runtime_state["recent_failures"] = recent_failures
    return runtime_state


def apply_failover_state(providers: list[dict], runtime_state: dict) -> list[dict]:
    recent_failures = runtime_state.get("recent_failures", {})
    for provider in providers:
        failure = recent_failures.get(provider["id"])
        provider["cooldown_until"] = failure.get("cooldown_until") if failure else None
        provider["degraded"] = failure is not None or (
            provider["installed"] and not provider["health_ready"] and provider["status"] not in {"disabled-by-override"}
        )
        provider["routing_ready"] = provider["health_ready"] and failure is None and not provider["disabled"]
        provider["ready"] = provider["routing_ready"]
        provider["routing_status"] = "cooldown" if failure and provider["health_ready"] else provider["status"]
        provider["routing_reason"] = (
            "Temporarily avoided because this agent failed recently."
            if failure and provider["health_ready"]
            else provider["status_reason"]
        )
    return providers


def best_ready_for_task(task: str, preferred: list[str], providers_by_id: dict[str, dict]) -> list[str]:
    picks = [provider_id for provider_id in preferred if providers_by_id.get(provider_id, {}).get("routing_ready")]
    if picks:
        return picks

    fallback = []
    for provider in providers_by_id.values():
        if provider.get("routing_ready") and task in provider.get("capability_hints", []):
            fallback.append(provider["id"])
    return sorted(fallback, key=lambda provider_id: (-providers_by_id[provider_id]["rank_score"], provider_id))


def build_best_available(providers: list[dict]) -> list[dict]:
    ready = [provider for provider in providers if provider["routing_ready"]]
    ready.sort(key=lambda item: (-item["rank_score"], item["display_name"].lower()))
    return [
        {
            "id": provider["id"],
            "display_name": provider["display_name"],
            "rank_score": provider["rank_score"],
            "best_hints": provider["capability_hints"][:4],
            "departments": provider["departments"],
        }
        for provider in ready[:5]
    ]


def build_best_by_task(tasks: dict, providers_by_id: dict[str, dict]) -> dict:
    results = {}
    for task, route in tasks.items():
        best = route["ready_candidates"][0] if route["ready_candidates"] else None
        alternatives = route["ready_candidates"][1:3]
        results[task] = {
            "best": best,
            "display_name": providers_by_id[best]["display_name"] if best else None,
            "alternatives": alternatives,
            "alternative_display_names": [providers_by_id[item]["display_name"] for item in alternatives],
        }
    return results


def is_healthy_snapshot(snapshot: dict) -> bool:
    required_departments = ["planning", "review", "verification"]
    for department in required_departments:
        candidates = snapshot["routing"]["departments"].get(department, {}).get("ready_candidates", [])
        if not candidates:
            return False
    return snapshot["summary"]["ready_agents"] > 0


def load_last_known_good(path: Path) -> dict | None:
    if not path.exists():
        return None
    return load_json(path)


def build_governance_snapshot(tasks_dir: Path = DEFAULT_TASKS_DIR) -> tuple[dict, list[str]]:
    records, warnings = load_task_records(tasks_dir)
    status_counts: dict[str, int] = {}
    department_dashboard: dict[str, dict] = {}
    tasks: list[dict] = []

    for record in records:
        contract = record["contract"]
        status = record["status"] or {
            "status": "planned",
            "assigned_worker": None,
            "department": contract.get("department"),
            "owner": contract.get("owner"),
            "history": [],
        }
        verification = record["verification"]
        quality = record["quality"]
        merge = record["merge"]
        department = status.get("department") or contract.get("department") or "unknown"
        task_status = status.get("status", "planned")

        status_counts[task_status] = status_counts.get(task_status, 0) + 1
        dashboard = department_dashboard.setdefault(department, {"total": 0, "statuses": {}})
        dashboard["total"] += 1
        dashboard["statuses"][task_status] = dashboard["statuses"].get(task_status, 0) + 1

        tasks.append(
            {
                "task_id": record["task_id"],
                "contract": contract_summary(contract),
                "contract_errors": record["contract_errors"],
                "status": status,
                "verification": verification,
                "quality": quality,
                "merge": merge,
            }
        )

    return {
        "tasks_dir": str(tasks_dir),
        "tasks": tasks,
        "summary": {
            "tracked_tasks": len(tasks),
            "status_counts": status_counts,
        },
        "department_dashboard": department_dashboard,
    }, warnings


def build_last_known_good(snapshot: dict) -> dict:
    return {
        "version": snapshot["version"],
        "timestamp": snapshot["generated_at"],
        "summary": snapshot["summary"],
        "ready_agents": [
            {
                "id": provider["id"],
                "display_name": provider["display_name"],
                "rank_score": provider["rank_score"],
            }
            for provider in snapshot["providers"]
            if provider["routing_ready"]
        ],
        "routing": snapshot["routing"],
    }


def persist_operational_state(snapshot: dict) -> None:
    write_json(DEFAULT_RUNTIME_STATE, snapshot["operations"]["runtime_state"])
    if snapshot["operations"]["next_last_known_good"]:
        write_json(DEFAULT_LAST_KNOWN_GOOD, snapshot["operations"]["next_last_known_good"])


def build_snapshot() -> dict:
    current_time = now_utc()
    profile_doc = load_json(PROFILE_PATH)
    routing_doc = load_json(ROUTING_PATH)
    override_doc, warnings = load_operator_json(
        DEFAULT_OVERRIDE,
        {
            "version": VERSION,
            "providers": {},
            "routing": {"tasks": {}, "departments": {}},
            "runtime": {"cooldown_seconds": DEFAULT_COOLDOWN_SECONDS},
        },
        "provider override",
    )
    profile_doc, routing_doc = apply_overrides(profile_doc, routing_doc, override_doc)

    runtime_state = prune_runtime_state(load_runtime_state(DEFAULT_RUNTIME_STATE, override_doc), current_time)
    providers = [discover_provider(provider) for provider in profile_doc["providers"]]
    runtime_state = update_runtime_state(runtime_state, providers, current_time)
    providers = apply_failover_state(providers, runtime_state)

    providers_with_rank = []
    for provider in providers:
        provider["rank_score"] = compute_rank(provider, provider["status"])
        providers_with_rank.append(provider)

    providers_with_rank.sort(key=lambda item: (-item["rank_score"], item["display_name"].lower()))
    providers_by_id = {provider["id"]: provider for provider in providers_with_rank}

    installed = [provider for provider in providers_with_rank if provider["installed"]]
    ready = [provider for provider in providers_with_rank if provider["routing_ready"]]
    broken = [provider for provider in installed if not provider["health_ready"]]
    degraded = [provider for provider in providers_with_rank if provider["degraded"]]

    tasks = {}
    for task, route in routing_doc["task_routing"].items():
        tasks[task] = {
            "preferred_order": route["preferred"],
            "ready_candidates": best_ready_for_task(task, route["preferred"], providers_by_id),
            "fallback_policy": route["fallback"],
        }

    departments = {}
    for department, preferred in routing_doc["departments"].items():
        if department == "fallback":
            departments[department] = {
                "preferred_order": [],
                "ready_candidates": [provider["id"] for provider in ready],
            }
            continue
        departments[department] = {
            "preferred_order": preferred,
            "ready_candidates": [provider_id for provider_id in preferred if providers_by_id.get(provider_id, {}).get("routing_ready")],
        }

    best_available_workers = build_best_available(providers_with_rank)
    best_by_task = build_best_by_task(tasks, providers_by_id)
    previous_last_known_good = load_last_known_good(DEFAULT_LAST_KNOWN_GOOD)
    governance, governance_warnings = build_governance_snapshot()
    warnings.extend(governance_warnings)

    snapshot = {
        "version": VERSION,
        "generated_at": current_time.isoformat(),
        "repo_root": str(REPO_ROOT),
        "cache_path": str(DEFAULT_CACHE),
        "report_path": str(DEFAULT_REPORT),
        "warnings": warnings,
        "summary": {
            "known_providers": len(profile_doc["providers"]),
            "installed_clis": len(installed),
            "ready_agents": len(ready),
            "broken_agents": len(broken),
            "degraded_agents": len(degraded),
        },
        "codex_role": routing_doc["codex_role"],
        "providers": providers_with_rank,
        "routing": {
            "tasks": tasks,
            "departments": departments,
            "best_available_workers": best_available_workers,
            "best_by_task": best_by_task,
        },
        "operations": {
            "override_path": str(DEFAULT_OVERRIDE),
            "runtime_state_path": str(DEFAULT_RUNTIME_STATE),
            "last_known_good_path": str(DEFAULT_LAST_KNOWN_GOOD),
            "tasks_path": str(DEFAULT_TASKS_DIR),
            "cooldown_seconds": runtime_state["cooldown_seconds"],
            "recently_failed_agents": [
                {
                    "id": provider_id,
                    "reason": record["reason"],
                    "cooldown_until": record["cooldown_until"],
                }
                for provider_id, record in sorted(runtime_state["recent_failures"].items())
            ],
            "degraded_agents": [provider["id"] for provider in degraded],
            "runtime_state": runtime_state,
            "next_last_known_good": None,
        },
        "last_known_good": previous_last_known_good,
        "governance": governance,
    }

    if is_healthy_snapshot(snapshot):
        next_last_known_good = build_last_known_good(snapshot)
        snapshot["last_known_good"] = next_last_known_good
        snapshot["operations"]["next_last_known_good"] = next_last_known_good

    return snapshot


def render_table(snapshot: dict) -> str:
    header = f"{'Agent':24} {'Status':24} {'Command':18} {'Hints'}"
    lines = [header, "-" * len(header)]
    for provider in snapshot["providers"]:
        hints = ", ".join(provider["capability_hints"][:3])
        command = provider["command_found"] or (provider["commands"][0] if provider["commands"] else "-")
        lines.append(f"{provider['display_name'][:24]:24} {provider['routing_status'][:24]:24} {command[:18]:18} {hints}")
    return "\n".join(lines)


def write_snapshot(snapshot: dict, dest: Path) -> None:
    ensure_parent(dest)
    dest.write_text(json.dumps(snapshot, indent=2), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Lightweight local agent discovery for together.")
    parser.add_argument("--format", choices=["json", "table"], default="table")
    parser.add_argument("--write", help="Optional path to write snapshot JSON")
    args = parser.parse_args()

    snapshot = build_snapshot()

    if args.write:
        write_snapshot(snapshot, Path(args.write))
        persist_operational_state(snapshot)

    if args.format == "json":
        json.dump(snapshot, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print(render_table(snapshot))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
