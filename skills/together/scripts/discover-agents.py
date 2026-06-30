#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = Path(__file__).resolve().parents[3]
DATA_DIR = SKILL_ROOT / "data"
PROFILE_PATH = DATA_DIR / "agent-profiles.json"
ROUTING_PATH = DATA_DIR / "capability-routing.json"
DEFAULT_CACHE = REPO_ROOT / ".together" / "cache" / "agent-registry.json"
DEFAULT_REPORT = REPO_ROOT / ".together" / "reports" / "agent-report.md"

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


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


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


def compute_rank(provider: dict, status: str) -> int:
    base = {
        "ready": 100,
        "installed-unknown": 60,
        "auth-required": 40,
        "permission-denied": 30,
        "installed-but-broken": 20,
        "not-installed": 0,
    }.get(status, 0)
    confidence_bonus = {
        "high": 8,
        "medium": 4,
        "low": 1,
    }.get(provider.get("confidence", "low"), 0)
    return base + min(len(provider.get("capability_hints", [])), 8) + confidence_bonus


def discover_provider(provider: dict) -> dict:
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
    ready = status == "ready"

    return {
        "id": provider["id"],
        "display_name": provider["display_name"],
        "commands": provider.get("commands", []),
        "command_found": found_command,
        "path": found_path,
        "installed": installed,
        "ready": ready,
        "status": status,
        "status_reason": status_reason,
        "checks": checks,
        "capability_hints": provider.get("capability_hints", []),
        "departments": provider.get("departments", []),
        "confidence": provider.get("confidence", "low"),
    }


def best_ready_for_task(task: str, preferred: list[str], providers_by_id: dict[str, dict]) -> list[str]:
    picks = [provider_id for provider_id in preferred if providers_by_id.get(provider_id, {}).get("ready")]
    if picks:
        return picks

    fallback = []
    for provider in providers_by_id.values():
        if provider.get("ready") and task in provider.get("capability_hints", []):
            fallback.append(provider["id"])
    return sorted(fallback, key=lambda provider_id: (-providers_by_id[provider_id]["rank_score"], provider_id))


def build_snapshot() -> dict:
    profile_doc = load_json(PROFILE_PATH)
    routing_doc = load_json(ROUTING_PATH)
    providers = [discover_provider(provider) for provider in profile_doc["providers"]]

    providers_with_rank = []
    for provider, source in zip(providers, profile_doc["providers"], strict=True):
        provider["rank_score"] = compute_rank(source, provider["status"])
        providers_with_rank.append(provider)

    providers_with_rank.sort(key=lambda item: (-item["rank_score"], item["display_name"].lower()))
    providers_by_id = {provider["id"]: provider for provider in providers_with_rank}

    installed = [provider for provider in providers_with_rank if provider["installed"]]
    ready = [provider for provider in providers_with_rank if provider["ready"]]
    broken = [provider for provider in installed if not provider["ready"]]

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
            "ready_candidates": [provider_id for provider_id in preferred if providers_by_id.get(provider_id, {}).get("ready")],
        }

    return {
        "version": "0.2.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "repo_root": str(REPO_ROOT),
        "cache_path": str(DEFAULT_CACHE),
        "report_path": str(DEFAULT_REPORT),
        "summary": {
            "known_providers": len(profile_doc["providers"]),
            "installed_clis": len(installed),
            "ready_agents": len(ready),
            "broken_agents": len(broken),
        },
        "codex_role": routing_doc["codex_role"],
        "providers": providers_with_rank,
        "routing": {
            "tasks": tasks,
            "departments": departments,
        },
    }


def render_table(snapshot: dict) -> str:
    header = f"{'Agent':24} {'Status':24} {'Command':18} {'Hints'}"
    lines = [header, "-" * len(header)]
    for provider in snapshot["providers"]:
        hints = ", ".join(provider["capability_hints"][:3])
        command = provider["command_found"] or (provider["commands"][0] if provider["commands"] else "-")
        lines.append(f"{provider['display_name'][:24]:24} {provider['status'][:24]:24} {command[:18]:18} {hints}")
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

    if args.format == "json":
        json.dump(snapshot, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print(render_table(snapshot))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
