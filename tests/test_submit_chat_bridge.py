from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "skills" / "together" / "scripts" / "submit-chat.py"


def load_module():
    spec = importlib.util.spec_from_file_location("submit_chat", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


def test_bridge_builds_codex_app_chat_command() -> None:
    module = load_module()

    command = module.build_submit_command(Path("together.exe"), "make a landing page")

    assert command == [
        "together.exe",
        "chat",
        "--source",
        "codex-app",
        "make a landing page",
    ]


def test_bridge_reports_missing_binary_without_traceback(tmp_path: Path) -> None:
    module = load_module()

    result = module.submit_chat("hello", search_roots=[tmp_path], env_path="")

    assert result.returncode == 2
    assert "Together binary not found" in result.stderr
    assert "Traceback" not in result.stderr
