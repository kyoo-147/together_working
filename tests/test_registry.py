from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from together.registry import REQUIRED_PROVIDER_FIELDS, load_profiles, validate_provider_profiles


def test_registry_json_parses() -> None:
    profiles = load_profiles()
    assert isinstance(profiles["providers"], list)
    assert profiles["providers"]


def test_required_provider_fields_exist() -> None:
    profiles = load_profiles()
    for provider in profiles["providers"]:
        assert REQUIRED_PROVIDER_FIELDS <= set(provider)


def test_registry_validation_passes() -> None:
    assert validate_provider_profiles(load_profiles()) == []

