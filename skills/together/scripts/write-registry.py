#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", nargs="?", default="")
    args = parser.parse_args()

    script = Path(__file__).with_name("discover-agents.py")
    output = Path(args.output) if args.output else Path(__file__).resolve().parents[3] / ".together" / "cache" / "agent-registry.json"
    proc = subprocess.run(
        [sys.executable, str(script), "--format", "json", "--write", str(output)],
        stdout=subprocess.DEVNULL,
        check=False,
    )
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
