# Command Code Commands

Useful low-cost checks:

```bash
cmdc --help
cmdc --version
cmdc --list-models
cmdc status
```

Useful non-interactive patterns:

```bash
cmdc -p "Summarize this repo" --model claude-sonnet-4-6
cmdc -p "Review this diff" --model gpt-5.3-codex
```

Use `cmdc` when:
- model choice matters
- vision-capable or long-context models help
- review should be separated from implementation
