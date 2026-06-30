# Antigravity Commands

Useful low-cost checks:

```bash
agy --help
agy --version
agy models
```

Useful bounded patterns:

```bash
agy -p "Audit this repo and summarize risks" --add-dir .
agy -p "Update only file X and report changes" --add-dir .
```

Use `agy` when:
- the task can be tightly scoped
- a sidecar worker is enough
- parallel bounded work is useful
