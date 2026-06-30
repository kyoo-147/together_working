# Architecture

`together` has one job: keep AI department work clean.

Control model:
- Codex owns planning, coordination, verification, integration
- workers stay replaceable
- registry stays static and editable
- runtime state stays local and cheap
- governance stays explicit

Flow:
1. load known-provider registry
2. scan PATH for installed CLIs
3. run lightweight checks
4. classify ready vs broken
5. apply override and cooldown failover rules
6. build department routing
7. verify scope and contract
8. write cache + report
9. let Codex choose merge outcome

Design bias:
- short
- explicit
- low token
- low magic
