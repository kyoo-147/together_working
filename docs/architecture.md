# Architecture

`together` has one job: keep orchestration clean.

Control model:
- Codex owns planning, decomposition, review gate, integration
- workers stay replaceable
- registry stays static and editable
- runtime state stays local and cheap

Flow:
1. load known-provider registry
2. scan PATH for installed CLIs
3. run lightweight checks
4. classify ready vs broken
5. build task routing and department view
6. write cache + report
7. let Codex choose execution batches

Design bias:
- short
- explicit
- low token
- low magic
