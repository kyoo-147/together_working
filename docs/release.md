# Release

Release checklist:
- update docs and examples
- run doctor
- run validation scripts
- run tests
- ensure runtime files are ignored
- verify no local paths or secrets are committed

Recommended commands:

```bash
python -m compileall skills/together/scripts scripts src tests
python skills/together/scripts/doctor.py
python scripts/validate-json.py
python scripts/validate-registry.py
python scripts/validate-routing.py
pytest
```

