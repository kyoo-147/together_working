# 04 — Domain Model

## Entities

### Workspace
- id
- root_path
- git_repository
- policies
- departments

### Department
- id
- name
- purpose
- capabilities
- primary_agents
- fallback_agents
- routing_policy

### Agent
- id
- provider
- executable
- version
- capabilities
- readiness
- load
- health
- config

### Task
- id
- intent
- contract
- department
- priority
- state
- parent_task
- created_at

### TaskContract
- scope
- inputs
- allowed_paths
- denied_paths
- constraints
- deliverables
- success_criteria
- review_policy
- verification_policy
- timeout
- retry
- merge_policy

### Execution
- id
- task_id
- agent_id
- pty_session_id
- attempt
- state
- started_at
- ended_at
- exit_code

### Review
- reviewer
- verdict
- comments
- findings
- diff_ref

### VerificationRun
- checks
- results
- evidence
- policy_decision

### Event
- sequence
- timestamp
- type
- actor
- payload

## State machines

### Agent
unknown → probing → ready ↔ busy → degraded → cooldown → ready/offline

### Task
queued → planning → executing → reviewing → verifying → ready_to_merge → merged

Error paths:
executing → blocked/failed → fallback → executing
reviewing → changes_requested → executing
verifying → failed → executing/cancelled

## Invariants
- Task đang execute phải có contract.
- Agent offline không được route.
- Verification required chưa pass thì không ready_to_merge.
- Merge phải tạo audit event.
- Fallback không được nới rộng file permissions tự động.
