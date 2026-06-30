#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


TEMPLATE = """# Resume Report

- objective: {objective}
- decomposition: {decomposition}
- completed: {completed}
- paused_at: {paused_at}
- attempted_routes: {attempted_routes}
- failure_summary: {failure_summary}
- limits_or_wait_states: {limits}
- next_recommended_batch: {next_batch}
- exact_resume_commands: {resume_commands}
- unresolved_risks: {risks}
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    parser.add_argument("--objective", default="")
    parser.add_argument("--decomposition", default="")
    parser.add_argument("--completed", default="")
    parser.add_argument("--paused-at", default="")
    parser.add_argument("--attempted-routes", default="")
    parser.add_argument("--failure-summary", default="")
    parser.add_argument("--limits", default="")
    parser.add_argument("--next-batch", default="")
    parser.add_argument("--resume-commands", default="")
    parser.add_argument("--risks", default="")
    args = parser.parse_args()

    text = TEMPLATE.format(
        objective=args.objective,
        decomposition=args.decomposition,
        completed=args.completed,
        paused_at=args.paused_at,
        attempted_routes=args.attempted_routes,
        failure_summary=args.failure_summary,
        limits=args.limits,
        next_batch=args.next_batch,
        resume_commands=args.resume_commands,
        risks=args.risks,
    )
    Path(args.output).write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
