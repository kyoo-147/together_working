from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.reporting import render_report


def test_report_renderer_can_render_sample_data() -> None:
    snapshot = json.loads((ROOT / "examples" / "agent-registry.json").read_text(encoding="utf-8"))
    report = render_report(snapshot)
    assert "# Together Agent Report" in report
    assert "## Summary" in report
    assert "## Last Known Good" in report

