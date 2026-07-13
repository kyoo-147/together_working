#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]


class BridgeResult:
    def __init__(self, returncode: int, stdout: str = "", stderr: str = "") -> None:
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def build_submit_command(binary: Path, text: str) -> list[str]:
    return [str(binary), "chat", "--source", "codex-app", text]


def find_together_binary(search_roots: list[Path] | None = None, env_path: str | None = None) -> Path | None:
    roots = search_roots or [REPO_ROOT, Path.cwd()]
    candidates: list[Path] = []
    for root in roots:
        candidates.extend(
            [
                root / "target" / "debug" / "together.exe",
                root / "target" / "release" / "together.exe",
                root / "bin" / "together.exe",
            ]
        )

    local_app = os.environ.get("LOCALAPPDATA")
    if local_app:
        candidates.append(Path(local_app) / "Together" / "together.exe")

    for candidate in candidates:
        if candidate.exists():
            return candidate

    path_value = os.environ.get("PATH", "") if env_path is None else env_path
    for entry in path_value.split(os.pathsep):
        if not entry:
            continue
        candidate = Path(entry) / "together.exe"
        if candidate.exists():
            return candidate
    return None


def submit_chat(
    text: str,
    search_roots: list[Path] | None = None,
    env_path: str | None = None,
) -> BridgeResult:
    binary = find_together_binary(search_roots=search_roots, env_path=env_path)
    if binary is None:
        return BridgeResult(
            2,
            stderr=(
                "Together binary not found. Run `cargo build -p cli` from the repo, "
                "install the release binary, or add together.exe to PATH."
            ),
        )

    proc = subprocess.run(
        build_submit_command(binary, text),
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    return BridgeResult(proc.returncode, proc.stdout, proc.stderr)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Submit Codex app chat into Together daemon.")
    parser.add_argument("text", help="Request text to send to Together.")
    args = parser.parse_args(argv)

    result = submit_chat(args.text)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
