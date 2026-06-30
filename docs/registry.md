# Registry

Registry holds known providers, not only installed CLIs.

Three layers matter:
- Known Providers: curated ecosystem coverage
- Installed CLIs: commands found on PATH
- Ready Agents: installed CLIs that pass lightweight checks

Operations layer adds:
- Last Known Good cache
- runtime failover memory
- machine-local provider override

Override file:
- `.together/providers.override.json`

Supported controls:
- `providers.<id>.disabled`
- `providers.<id>.rank_adjust`
- `providers.<id>.disable_capabilities`
- `providers.<id>.add_capabilities`
- `routing.tasks.<task>.preferred_first`
- `routing.tasks.<task>.preferred_last`
- `routing.tasks.<task>.remove_preferred`
- same three keys for `routing.departments.<name>`
- `runtime.cooldown_seconds`

Capability hints are conservative:
- `vision`
- `backend`
- `frontend`
- `research`
- `review`
- `verification`
- `docs`
- `shell`
- `short_task`
- `long_task`
- `multi_file`

Hints are routing input, not benchmark output.
