from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DISCOVER = ROOT / "skills" / "together" / "scripts" / "discover-agents.py"


def load_module():
    spec = importlib.util.spec_from_file_location("together_discover_agents_test", DISCOVER)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


def test_override_parser_handles_valid_json(tmp_path: Path) -> None:
    module = load_module()
    path = tmp_path / "override.json"
    path.write_text(json.dumps({"version": "0.5.0", "providers": {}}), encoding="utf-8")
    data, warnings = module.load_operator_json(path, {"providers": {}}, "provider override")
    assert warnings == []
    assert data["version"] == "0.5.0"


def test_override_parser_handles_utf8_bom(tmp_path: Path) -> None:
    module = load_module()
    path = tmp_path / "override.json"
    path.write_bytes(("\ufeff" + json.dumps({"version": "0.5.0", "providers": {}})).encode("utf-8"))
    data, warnings = module.load_operator_json(path, {"providers": {}}, "provider override")
    assert warnings == []
    assert data["version"] == "0.5.0"


def test_override_parser_handles_malformed_json(tmp_path: Path) -> None:
    module = load_module()
    path = tmp_path / "override.json"
    path.write_text("{\n  \"version\": \"0.5.0\",\n", encoding="utf-8")
    data, warnings = module.load_operator_json(path, {"providers": {}}, "provider override")
    assert data == {"providers": {}}
    assert warnings
    assert "Ignored provider override" in warnings[0]
