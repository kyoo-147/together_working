# doctor

Purpose:
- generate full operational snapshot and report

When to use:
- before major work
- before release checks

Inputs:
- provider catalog
- local machine state
- override config

Outputs:
- registry cache
- runtime state
- report

Safety rules:
- keep checks lightweight

Example:

```bash
python skills/together/scripts/doctor.py
```

