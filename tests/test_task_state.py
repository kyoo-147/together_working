from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.task_state import create_task_state, transition_task_state


def test_task_state_transition_records_history() -> None:
    state = create_task_state(task_id="TASK-123", department="engineering", owner="codex")
    next_state = transition_task_state(state, "assigned", worker="cmdc")
    assert next_state["status"] == "assigned"
    assert next_state["assigned_worker"] == "cmdc"
    assert next_state["history"][-1]["status"] == "assigned"


def test_task_state_rejects_invalid_transition() -> None:
    state = create_task_state(task_id="TASK-123", department="engineering", owner="codex")
    with pytest.raises(ValueError):
        transition_task_state(state, "integrated")
