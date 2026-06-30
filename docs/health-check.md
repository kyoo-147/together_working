# Health Check

Health check stays lightweight.

Allowed:
- find command on PATH
- run `--help`
- run `--version`
- run cheap auth/status command when known safe
- inspect obvious auth or permission errors

Not allowed:
- calling a model
- prompting big tasks
- benchmarking
- spending meaningful tokens

States:
- `not-installed`
- `ready`
- `auth-required`
- `permission-denied`
- `installed-but-broken`
- `installed-unknown`
