from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.contracts import REQUIRED_CONTRACT_FIELDS, load_contract, validate_contract


def test_contract_example_loads() -> None:
    contract = load_contract(ROOT / "examples" / "task-contract.example.yaml")
    assert REQUIRED_CONTRACT_FIELDS <= set(contract)
    assert contract["task_id"] == "TASK-123"
    assert contract["reviewer_required"] is True


def test_contract_validation_catches_missing_fields() -> None:
    errors = validate_contract({"task_id": "TASK-999"})
    assert errors
    assert any("missing fields" in error for error in errors)
