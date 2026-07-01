#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "src"))

from together.task_validation import validate_task


def load_changed_files(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8").strip()
    if not text:
        return []
    if path.suffix.lower() == ".json":
        payload = json.loads(text)
        if isinstance(payload, dict):
            return list(payload.get("changed_files", []))
        if isinstance(payload, list):
            return list(payload)
        raise ValueError("Changed files JSON must be an object with changed_files or a list.")
    return [line.strip() for line in text.splitlines() if line.strip()]


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate one Together task contract end-to-end.")
    parser.add_argument("contract", help="Path to .contract.yaml file")
    parser.add_argument("--changed-files", help="Optional path to changed files payload")
    parser.add_argument("--base", help="Optional git ref")
    parser.add_argument("--staged", action="store_true", help="Use staged changes")
    parser.add_argument("--mode", choices=["warn", "strict"], help="Override enforcement mode")
    parser.add_argument("--write-artifacts", action="store_true", help="Write task artifacts under .together/tasks")
    args = parser.parse_args()

    changed_files = None
    if args.changed_files:
        try:
            changed_files = load_changed_files(Path(args.changed_files))
        except Exception as exc:
            json.dump({"error": "invalid-changed-files", "message": str(exc)}, sys.stdout, indent=2)
            sys.stdout.write("\n")
            return 2

    result, exit_code = validate_task(
        Path(args.contract),
        cwd=Path.cwd(),
        changed_files=changed_files,
        staged=args.staged,
        base=args.base,
        mode=args.mode,
        write_artifacts=args.write_artifacts,
    )
    json.dump(result, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
