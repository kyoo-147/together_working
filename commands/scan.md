# scan

Purpose:
- detect known providers, installed CLIs, and ready agents

When to use:
- before routing work
- after installing or removing a CLI

Inputs:
- local PATH
- provider catalog

Outputs:
- terminal table or JSON snapshot

Safety rules:
- lightweight checks only
- no model call

Example:

```bash
python skills/together/scripts/discover-agents.py --format table
```

