#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "src"))

from together.benchmarking import compare_results


def main() -> int:
    parser = argparse.ArgumentParser(description="Compare codex_only and together benchmark results.")
    parser.add_argument("codex_only_result", help="Path to codex_only result JSON")
    parser.add_argument("together_result", help="Path to together result JSON")
    parser.add_argument("--output", help="Optional markdown output path")
    args = parser.parse_args()

    codex_only = json.loads(Path(args.codex_only_result).read_text(encoding="utf-8"))
    together = json.loads(Path(args.together_result).read_text(encoding="utf-8"))
    report = compare_results(codex_only, together)
    if args.output:
        Path(args.output).write_text(report, encoding="utf-8")
    sys.stdout.write(report)
    if not report.endswith("\n"):
        sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
