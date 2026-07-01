from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.verification_result import build_verification_result, validate_verification_result


def test_verification_result_has_all_checks() -> None:
    result = build_verification_result(task_id="TASK-123")
    assert result["status"] == "NEEDS_REVIEW"
    assert "scope_compliance" in result["checks"]
    assert all("status" in check and "evidence" in check and "notes" in check for check in result["checks"].values())


def test_verification_result_validation_rejects_bad_check_shape() -> None:
    result = build_verification_result(task_id="TASK-123")
    result["checks"]["scope_compliance"] = {"status": "PASS"}
    errors = validate_verification_result(result)
    assert errors
