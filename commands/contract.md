# contract

Purpose:
- define worker boundaries before execution

When to use:
- any scoped worker task

Inputs:
- task id
- scope
- allowed files
- denied files
- success criteria

Outputs:
- task contract

Safety rules:
- no worker should start without clear scope on important tasks

Example:
- see `examples/task-contract.example.yaml`

