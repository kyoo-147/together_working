#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def run(step: list[str]) -> int:
    print("$", " ".join(step))
    return subprocess.run(step, cwd=REPO_ROOT, check=False).returncode


def main() -> int:
    steps = [
        [sys.executable, "-m", "compileall", "skills/together/scripts", "scripts", "src", "tests"],
        [sys.executable, "skills/together/scripts/doctor.py"],
        [sys.executable, "scripts/validate-json.py"],
        [sys.executable, "scripts/validate-registry.py"],
        [sys.executable, "scripts/validate-routing.py"],
    ]
    for step in steps:
        code = run(step)
        if code != 0:
            return code
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

