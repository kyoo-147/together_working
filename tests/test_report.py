from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.reporting import render_report


def test_report_renderer_can_render_sample_data() -> None:
    snapshot = json.loads((ROOT / "examples" / "agent-registry.json").read_text(encoding="utf-8"))
    snapshot["governance"] = {
        "tasks": [
            {
                "task_id": "TASK-123",
                "contract": json.loads(
                    json.dumps(
                        {
                            "task_id": "TASK-123",
                            "title": "Harden routing fallback validation",
                            "department": "engineering",
                            "role": "implementer",
                            "owner": "codex",
                            "risk_level": "medium",
                            "enforcement_mode": "warn",
                            "allowed_files": ["src/together/routing.py"],
                            "denied_files": ["src/together/registry.py"],
                        }
                    )
                ),
                "status": json.loads((ROOT / "examples" / "task-status.example.json").read_text(encoding="utf-8")),
                "verification": json.loads((ROOT / "examples" / "verification.example.json").read_text(encoding="utf-8")),
                "quality": json.loads((ROOT / "examples" / "quality-gate.example.json").read_text(encoding="utf-8")),
                "merge": json.loads((ROOT / "examples" / "merge-decision.example.json").read_text(encoding="utf-8")),
                "contract_errors": [],
            }
        ],
        "department_dashboard": {
            "engineering": {
                "total": 1,
                "statuses": {
                    "passed": 1
                }
            }
        },
    }
    report = render_report(snapshot)
    assert "# Together Agent Report" in report
    assert "## Summary" in report
    assert "## Task Contracts" in report
    assert "## Verification Results" in report
    assert "## Quality Gates" in report
    assert "## Merge Decisions" in report
    assert "## Department Dashboard" in report
    assert "## Last Known Good" in report


def test_report_renderer_handles_missing_task_artifacts_cleanly() -> None:
    snapshot = json.loads((ROOT / "examples" / "agent-registry.json").read_text(encoding="utf-8"))
    snapshot["governance"] = {
        "tasks": [],
        "department_dashboard": {},
    }
    report = render_report(snapshot)
    assert "No task artifacts found." in report
