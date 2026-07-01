#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from together.registry import load_profiles
from together.routing import load_routing, validate_routing_config


def main() -> int:
    errors = validate_routing_config(load_profiles(), load_routing())
    if errors:
        print("Routing validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    print("Routing validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

