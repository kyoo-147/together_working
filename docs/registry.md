# Registry

Registry holds known providers, not only installed CLIs.

Three layers matter:
- Known Providers: curated ecosystem coverage
- Installed CLIs: commands found on PATH
- Ready Agents: installed CLIs that pass lightweight checks

Capability hints are conservative:
- `vision`
- `backend`
- `frontend`
- `research`
- `review`
- `docs`
- `shell`
- `short_task`
- `long_task`
- `multi_file`

Hints are routing input, not benchmark output.
