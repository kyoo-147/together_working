from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.registry import load_profiles
from together.routing import VALID_CAPABILITIES, VALID_DEPARTMENTS, VALID_TASKS, load_routing, validate_routing_config


def test_routing_json_parses() -> None:
    routing = load_routing()
    assert "task_routing" in routing
    assert "departments" in routing


def test_routing_references_valid_departments_and_capabilities() -> None:
    routing = load_routing()
    assert set(routing["task_routing"]).issubset(VALID_TASKS)
    assert set(routing["departments"]).issubset(VALID_DEPARTMENTS)


def test_routing_validation_passes() -> None:
    assert validate_routing_config(load_profiles(), load_routing()) == []
