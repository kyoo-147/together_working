#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path


DISCOVER_PATH = Path(__file__).with_name("discover-agents.py")
SPEC = importlib.util.spec_from_file_location("together_discover_agents", DISCOVER_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(MODULE)

DEFAULT_CACHE = MODULE.DEFAULT_CACHE
DEFAULT_REPORT = MODULE.DEFAULT_REPORT


def main() -> int:
    steps = [
        [sys.executable, str(Path(__file__).with_name("discover-agents.py")), "--format", "json", "--write", str(DEFAULT_CACHE)],
        [sys.executable, str(Path(__file__).with_name("render-report.py")), "--input", str(DEFAULT_CACHE), "--output", str(DEFAULT_REPORT)],
    ]
    for step in steps:
        proc = subprocess.run(step, check=False, stdout=subprocess.DEVNULL)
        if proc.returncode != 0:
            return proc.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
