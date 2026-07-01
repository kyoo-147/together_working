# Contributing

## Add a provider

1. Edit `skills/together/data/agent-profiles.json`
2. Add `id`, `display_name`, `commands`, `lightweight_checks`, `capability_hints`, `departments`, `confidence`
3. Keep hints conservative

## Add capability hints

- use only existing routing hints unless introducing a justified new capability
- update both provider profiles and routing validation if capability set changes

## Update routing

1. Edit `skills/together/data/capability-routing.json`
2. Keep provider ids valid
3. Keep Verification department intact

## Run validation

```bash
python -m compileall skills/together/scripts scripts src tests
python skills/together/scripts/doctor.py
python scripts/validate-json.py
python scripts/validate-registry.py
python scripts/validate-routing.py
pytest
```

## PR checklist

- [ ] task scope stayed focused
- [ ] docs updated if behavior changed
- [ ] generated runtime files not committed
- [ ] validation passed
- [ ] final integration path still belongs to Codex

