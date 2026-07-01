# report

Purpose:
- render human-readable operational report from registry snapshot

When to use:
- after doctor
- when sharing current system state

Inputs:
- snapshot JSON

Outputs:
- markdown report

Safety rules:
- scrub secrets before publishing examples

Example:

```bash
python skills/together/scripts/render-report.py
```

