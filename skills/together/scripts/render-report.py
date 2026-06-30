#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path


DISCOVER_PATH = Path(__file__).with_name("discover-agents.py")
SPEC = importlib.util.spec_from_file_location("together_discover_agents", DISCOVER_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(MODULE)

DEFAULT_CACHE = MODULE.DEFAULT_CACHE
DEFAULT_REPORT = MODULE.DEFAULT_REPORT
build_snapshot = MODULE.build_snapshot
ensure_parent = MODULE.ensure_parent


def load_snapshot(path: Path | None) -> dict:
    if path and path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    return build_snapshot()


def join_or_dash(values: list[str], limit: int = 4) -> str:
    if not values:
        return "-"
    return ", ".join(values[:limit])


def render_summary(snapshot: dict) -> list[str]:
    summary = snapshot["summary"]
    return [
        "## Summary",
        "",
        "| Metric | Count |",
        "|---|---:|",
        f"| Known Providers | {summary['known_providers']} |",
        f"| Installed CLIs | {summary['installed_clis']} |",
        f"| Ready Agents | {summary['ready_agents']} |",
        f"| Broken Agents | {summary['broken_agents']} |",
        f"| Degraded Agents | {summary['degraded_agents']} |",
        "",
    ]


def render_known(snapshot: dict) -> list[str]:
    lines = [
        "## Known Providers",
        "",
        "| Provider | Commands | Hint Profile |",
        "|---|---|---|",
    ]
    for provider in sorted(snapshot["providers"], key=lambda item: item["display_name"].lower()):
        lines.append(
            f"| {provider['display_name']} | `{join_or_dash(provider['commands'], 3)}` | {join_or_dash(provider['capability_hints'], 4)} |"
        )
    lines.append("")
    return lines


def render_installed(snapshot: dict) -> list[str]:
    installed = [provider for provider in snapshot["providers"] if provider["installed"]]
    lines = [
        "## Installed CLIs",
        "",
        "| Agent | Status | Command | Path |",
        "|---|---|---|---|",
    ]
    if not installed:
        lines.append("| - | - | - | - |")
    else:
        for provider in installed:
            command = provider["command_found"] or join_or_dash(provider["commands"], 1)
            lines.append(f"| {provider['display_name']} | {provider['status']} | `{command}` | `{provider['path']}` |")
    lines.append("")
    return lines


def render_ready(snapshot: dict) -> list[str]:
    ready = [provider for provider in snapshot["providers"] if provider["ready"]]
    lines = [
        "## Ready Agents",
        "",
        "| Agent | Best Hints | Departments |",
        "|---|---|---|",
    ]
    if not ready:
        lines.append("| - | - | - |")
    else:
        for provider in ready:
            lines.append(
                f"| {provider['display_name']} | {join_or_dash(provider['capability_hints'], 4)} | {join_or_dash(provider['departments'], 3)} |"
            )
    lines.append("")
    return lines


def render_broken(snapshot: dict) -> list[str]:
    broken = [provider for provider in snapshot["providers"] if provider["installed"] and not provider["health_ready"]]
    lines = [
        "## Broken Agents",
        "",
        "| Agent | Status | Reason |",
        "|---|---|---|",
    ]
    if not broken:
        lines.append("| - | - | - |")
    else:
        for provider in broken:
            lines.append(f"| {provider['display_name']} | {provider['status']} | {provider['status_reason']} |")
    lines.append("")
    return lines


def render_best_available(snapshot: dict) -> list[str]:
    workers = snapshot["routing"]["best_available_workers"]
    lines = [
        "## Best Available Workers",
        "",
        "| Rank | Agent | Best Hints | Departments |",
        "|---:|---|---|---|",
    ]
    if not workers:
        lines.append("| 1 | - | - | - |")
    else:
        for index, worker in enumerate(workers, start=1):
            lines.append(
                f"| {index} | {worker['display_name']} | {join_or_dash(worker['best_hints'])} | {join_or_dash(worker['departments'])} |"
            )
    lines.append("")
    return lines


def render_best_by_task(snapshot: dict) -> list[str]:
    lines = [
        "## Best By Task",
        "",
        "| Task | Best Worker | Alternatives |",
        "|---|---|---|",
    ]
    for task, details in snapshot["routing"]["best_by_task"].items():
        lines.append(
            f"| {task} | {details['display_name'] or '-'} | {join_or_dash(details['alternative_display_names'])} |"
        )
    lines.append("")
    return lines


def render_degraded(snapshot: dict) -> list[str]:
    degraded = [provider for provider in snapshot["providers"] if provider["degraded"]]
    lines = [
        "## Degraded Agents",
        "",
        "| Agent | Routing Status | Cooldown Until | Reason |",
        "|---|---|---|---|",
    ]
    if not degraded:
        lines.append("| - | - | - | - |")
    else:
        for provider in degraded:
            lines.append(
                f"| {provider['display_name']} | {provider['routing_status']} | {provider['cooldown_until'] or '-'} | {provider['routing_reason']} |"
            )
    lines.append("")
    return lines


def render_last_known_good(snapshot: dict) -> list[str]:
    record = snapshot.get("last_known_good")
    lines = [
        "## Last Known Good",
        "",
    ]
    if not record:
        lines.extend(["No healthy snapshot stored yet.", ""])
        return lines

    summary = record["summary"]
    ready_agents = [item["display_name"] for item in record.get("ready_agents", [])]
    lines.extend(
        [
            f"- Timestamp: `{record['timestamp']}`",
            f"- Ready Agents: {summary['ready_agents']}",
            f"- Broken Agents: {summary['broken_agents']}",
            f"- Degraded Agents: {summary.get('degraded_agents', 0)}",
            f"- Workers: {join_or_dash(ready_agents, 6)}",
            "",
        ]
    )
    return lines


def render_tasks(snapshot: dict) -> list[str]:
    lines = [
        "## Task Routing",
        "",
        "| Task | Preferred Ready | Fallback Policy |",
        "|---|---|---|",
    ]
    for task, route in snapshot["routing"]["tasks"].items():
        lines.append(
            f"| {task} | {join_or_dash(route['ready_candidates'])} | {route['fallback_policy']} |"
        )
    lines.append("")
    return lines


def render_departments(snapshot: dict) -> list[str]:
    lines = [
        "## Department View",
        "",
        "| Department | Preferred Order | Ready Workers |",
        "|---|---|---|",
    ]
    for department, route in snapshot["routing"]["departments"].items():
        lines.append(
            f"| {department} | {join_or_dash(route['preferred_order'])} | {join_or_dash(route['ready_candidates'])} |"
        )
    lines.append("")
    return lines


def render_report(snapshot: dict) -> str:
    lines = [
        "# Together Agent Report",
        "",
        f"Generated: `{snapshot['generated_at']}`",
        "",
        f"Codex role: {', '.join(snapshot['codex_role'])}.",
        "",
    ]
    sections = [
        render_summary(snapshot),
        render_known(snapshot),
        render_installed(snapshot),
        render_ready(snapshot),
        render_broken(snapshot),
        render_best_available(snapshot),
        render_best_by_task(snapshot),
        render_degraded(snapshot),
        render_tasks(snapshot),
        render_departments(snapshot),
        render_last_known_good(snapshot),
    ]
    for section in sections:
        lines.extend(section)
    return "\n".join(lines).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Render together agent report.")
    parser.add_argument("--input", help="Optional snapshot JSON path")
    parser.add_argument("--output", default=str(DEFAULT_REPORT), help="Markdown output path")
    args = parser.parse_args()

    snapshot = load_snapshot(Path(args.input) if args.input else None)
    output = Path(args.output)
    ensure_parent(output)
    output.write_text(render_report(snapshot), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
