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


ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "data"
PROFILES = DATA_DIR / "agent-profiles.json"
CAP_ROUTING = DATA_DIR / "capability-routing.json"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_check(command: str, found_path: str, raw_args: list[str]) -> tuple[bool, str]:
    cmd = [command, *raw_args]
    shell = False
    run_target: list[str] | str = cmd

    # Windows npm wrappers like .CMD often need shell dispatch to execute correctly.
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
            timeout=15,
            shell=shell,
            check=False,
        )
    except Exception as exc:
        return False, str(exc)

    output = (proc.stdout or "") + ("\n" + proc.stderr if proc.stderr else "")
    return proc.returncode == 0, output.strip()


def classify_status(found_path: str | None, checks: dict[str, dict[str, str | bool]]) -> str:
    if not found_path:
        return "not-found"

    failures = [name for name, result in checks.items() if not result["ok"]]
    outputs = "\n".join(str(result["output"]).lower() for result in checks.values())
    has_success = any(result["ok"] for result in checks.values())
    auth_fail_markers = [
        "not configured",
        "not logged in",
        "not authenticated",
        "login required",
        "please login",
        "please log in",
        "authenticate first",
    ]
    auth_ready_markers = [
        "authenticated as",
        "authentication verified",
        "logged in as",
    ]

    if any(marker in outputs for marker in auth_fail_markers):
        return "installed-but-not-configured"
    if has_success and any(marker in outputs for marker in auth_ready_markers):
        return "ready"
    if has_success:
        return "ready"
    if failures and len(failures) == len(checks):
        return "installed-but-failing"
    return "unknown"


def compute_rank(status: str, capabilities: list[str]) -> int:
    base = {
        "ready": 100,
        "unknown": 60,
        "installed-but-not-configured": 40,
        "installed-but-failing": 20,
        "not-found": 0,
    }.get(status, 0)
    return base + min(len(capabilities), 8)


def discover() -> dict:
    profiles = load_json(PROFILES)["agents"]
    capability_pref = load_json(CAP_ROUTING)["capabilities"]

    registry = []
    for profile in profiles:
        commands = profile["commands"]
        found_command = None
        found_path = None
        for candidate in commands:
            resolved = shutil.which(candidate)
            if resolved:
                found_command = candidate
                found_path = resolved
                break

        checks: dict[str, dict[str, str | bool]] = {}
        if found_command:
            for raw in profile.get("preferred_checks", []):
                label = " ".join(raw)
                ok, output = run_check(found_command, found_path, raw)
                checks[label] = {"ok": ok, "output": output[:1200]}

        status = classify_status(found_path, checks)
        capabilities = profile.get("capabilities", [])
        recommended = profile.get("recommended_tasks", [])

        preferred_capabilities = []
        for cap, agents in capability_pref.items():
            if profile["id"] in agents:
                preferred_capabilities.append(cap)

        registry.append(
            {
                "id": profile["id"],
                "display_name": profile["display_name"],
                "command": found_command or commands[0],
                "path": found_path,
                "status": status,
                "checks": checks,
                "capabilities": capabilities,
                "preferred_capabilities": preferred_capabilities,
                "strengths": profile.get("strengths", []),
                "weaknesses": profile.get("weaknesses", []),
                "recommended_tasks": recommended,
                "rank_score": compute_rank(status, capabilities),
            }
        )

    registry.sort(key=lambda item: (-item["rank_score"], item["display_name"].lower()))
    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": str(PROFILES),
        "agents": registry,
    }


def print_table(snapshot: dict) -> None:
    header = f"{'Agent':24} {'Status':30} {'Command':18} {'Top capabilities'}"
    print(header)
    print("-" * len(header))
    for item in snapshot["agents"]:
        caps = ", ".join(item["preferred_capabilities"][:3] or item["capabilities"][:3])
        print(f"{item['display_name'][:24]:24} {item['status'][:30]:30} {item['command'][:18]:18} {caps}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Discover local AI agent CLIs for together.")
    parser.add_argument("--format", choices=["json", "table"], default="table")
    parser.add_argument("--write", help="Optional path to write the snapshot JSON")
    args = parser.parse_args()

    snapshot = discover()

    if args.write:
        dest = Path(args.write)
        dest.write_text(json.dumps(snapshot, indent=2), encoding="utf-8")

    if args.format == "json":
        json.dump(snapshot, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print_table(snapshot)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
