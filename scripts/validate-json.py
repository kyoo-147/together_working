#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
JSON_TARGETS = [
    REPO_ROOT / "skills" / "together" / "data",
    REPO_ROOT / "examples",
    REPO_ROOT / "examples" / "tasks",
    REPO_ROOT / ".together" / "providers.override.json",
    REPO_ROOT / ".together" / "providers.override.example.json",
    REPO_ROOT / ".claude-plugin" / "plugin.json",
    REPO_ROOT / ".claude-plugin" / "marketplace.json",
    REPO_ROOT / "package.json",
]


def iter_json_files():
    for target in JSON_TARGETS:
        if target.is_dir():
            yield from sorted(target.rglob("*.json"))
        elif target.exists():
            yield target


def main() -> int:
    errors: list[str] = []
    for path in iter_json_files():
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"{path.relative_to(REPO_ROOT)}: {exc}")
    if errors:
        print("JSON validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    print("JSON validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
