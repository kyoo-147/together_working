# validate

Purpose:
- run lightweight local release checks

When to use:
- before commit
- before push

Inputs:
- current repo

Outputs:
- pass/fail validation signals

Safety rules:
- do not mutate source unless command explicitly does so

Example:

```bash
python scripts/release-check.py
```

