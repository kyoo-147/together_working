# Install

## Install Together terminal app on Windows

From a fresh clone:

```powershell
git clone https://github.com/kyoo-147/together_working.git
cd together_working
powershell -ExecutionPolicy Bypass -File scripts\install.ps1 -AddToPath
```

Open a new terminal and run:

```powershell
together
```

If you do not want PATH changes:

```powershell
target\release\together.exe
```

## Verify install

```powershell
together doctor
together status --json
together self-check
```

## Codex skill bridge

After the binary is installed or built, Codex can submit work into the local daemon through:

```powershell
python skills\together\scripts\submit-chat.py "create a scoped task for README polish"
```

The Together TUI will show the same chat/proposal/task events.

## Install legacy skill package

```bash
npx skills add https://github.com/kyoo-147/together_working
```

Install main skill only:

```bash
npx skills add https://github.com/kyoo-147/together_working --skill "together"
```

## Quick commands

These Python commands are compatibility tools for scanner/report workflows.

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

Changed files:

```bash
python scripts/changed-files.py --json
```

Validate task:

```bash
python scripts/validate-task.py examples/task-contract.example.yaml --mode warn
python scripts/validate-task.py examples/task-contract.example.yaml --mode strict --write-artifacts
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
- `.together/tasks/*.status.json`
- `.together/tasks/*.verification.json`
- `.together/tasks/*.quality.json`
- `.together/tasks/*.merge.json`

These are ignored and should not be committed.

## v0.5 scope

Included now:
- enforcement engine
- task validation script
- git diff changed-files helper
- warn and strict modes
- task artifacts
- report integration

Not included yet:
- autonomous task execution runner
- automatic agent sandbox
- automatic commit or PR creation
- distributed scheduler
