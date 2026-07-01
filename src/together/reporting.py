from __future__ import annotations

import importlib.util
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_REPORT_PATH = REPO_ROOT / "skills" / "together" / "scripts" / "render-report.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("together_render_report", RENDER_REPORT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


def render_report(snapshot: dict) -> str:
    return _load_module().render_report(snapshot)

