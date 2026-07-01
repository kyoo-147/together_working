# verify

Purpose:
- check scope, quality, routing, and acceptance criteria before integration

When to use:
- after worker execution
- before final merge

Inputs:
- worker output
- task contract

Outputs:
- PASS / REJECT / NEEDS_REVIEW

Safety rules:
- verification must stay independent from implementation

Example:
- confirm only allowed files changed and acceptance criteria passed

