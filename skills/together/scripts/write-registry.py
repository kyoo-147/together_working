#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    args = parser.parse_args()

    script = Path(__file__).with_name("discover-agents.py")
    output = Path(args.output)
    proc = subprocess.run(
        ["python", str(script), "--format", "json", "--write", str(output)],
        check=False,
    )
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
