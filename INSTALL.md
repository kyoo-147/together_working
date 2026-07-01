# Install

## Install skill

```bash
npx skills add https://github.com/kyoo-147/together_working
```

Install main skill only:

```bash
npx skills add https://github.com/kyoo-147/together_working --skill "together"
```

## Quick commands

Scan:

```bash
python skills/together/scripts/discover-agents.py --format table
```

Doctor:

```bash
python skills/together/scripts/doctor.py
```

Report:

```bash
python skills/together/scripts/render-report.py
```

## Override config

Editable file:

```text
.together/providers.override.json
```

Example template:

```text
.together/providers.override.example.json
examples/providers.override.example.json
```

## Generated files

Runtime output:
- `.together/cache/agent-registry.json`
- `.together/cache/last-known-good.json`
- `.together/cache/runtime-state.json`
- `.together/reports/agent-report.md`

These are ignored and should not be committed.

