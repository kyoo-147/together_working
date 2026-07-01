#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from together.registry import load_profiles, validate_provider_profiles


def main() -> int:
    errors = validate_provider_profiles(load_profiles())
    if errors:
        print("Registry validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    print("Registry validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

