#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "src"))

from together.git_tools import list_changed_files


def main() -> int:
    parser = argparse.ArgumentParser(description="List changed files for Together task validation.")
    parser.add_argument("--staged", action="store_true", help="Read staged changes instead of working tree changes.")
    parser.add_argument("--base", help="Compare against a git ref.")
    parser.add_argument("--json", action="store_true", help="Write JSON output.")
    parser.add_argument("--plain", action="store_true", help="Write one path per line.")
    args = parser.parse_args()

    payload, error = list_changed_files(Path.cwd(), staged=args.staged, base=args.base)
    if error:
        if args.plain:
            print(error["message"], file=sys.stderr)
        else:
            json.dump(error, sys.stdout, indent=2)
            sys.stdout.write("\n")
        return 2

    if args.plain and not args.json:
        for path in payload["changed_files"]:
            print(path)
        return 0

    json.dump(payload, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
