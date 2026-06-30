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

Runtime routing can still downgrade a healthy agent temporarily when:
- it failed recently
- it is inside cooldown
- local override disables it
