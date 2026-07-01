#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "src"))

from together.benchmarking import benchmark_task


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark one task in codex_only or together mode.")
    parser.add_argument("task", help="Path to benchmark task YAML")
    parser.add_argument("--mode", required=True, choices=["codex_only", "together"])
    parser.add_argument("--base", help="Optional git ref for diff collection")
    parser.add_argument("--staged", action="store_true", help="Use staged changes instead of working tree changes")
    parser.add_argument("--write-result", action="store_true", help="Write JSON and markdown benchmark artifacts")
    parser.add_argument("--manual-total-tokens", type=int, help="Optional manual token total")
    args = parser.parse_args()

    result, exit_code = benchmark_task(
        Path(args.task),
        mode=args.mode,
        cwd=Path.cwd(),
        staged=args.staged,
        base=args.base,
        write_result=args.write_result,
        manual_total_tokens=args.manual_total_tokens,
    )
    json.dump(result, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
