# 05 — Terminal UI Structure

## Nguyên tắc
- Information-dense nhưng không rối.
- Màu chỉ biểu diễn state/severity.
- Real PTY ở trung tâm.
- Domain navigation là workspace/department/task, không phải pane-first.
- Keyboard-first, mouse optional.

## Main layout
 
```text
┌ Workspace / Departments ┬ Task execution ───────────────┬ Agent pool ───────┐
│                         │                               │                   │
│ PROJECT                 │ TASK-142                      │ READY             │
│ together_working        │ Fix flaky auth tests          │ Codex        idle │
│                         │                               │ Claude Code  busy │
│ DEPARTMENTS             │ Department: Engineering       │ Gemini CLI   idle │
│ ● Planning       ready  │ Worker: Amp                   │ OpenCode     idle │
│ ● Research       ready  │ Reviewer: Codex               │ Amp          busy │
│ ● Engineering   active  │ Verifier: CMDC                │ CMDC      verify │
│ ● Review         ready  │                               │                   │
│ ● Verification  active  ├───────────────────────────────┤ DEGRADED          │
│ ○ Fallback     standby  │ REAL PTY                      │ Agy       cooldown│
│                         │                               │                   │
│ TASKS                   │ $ amp                         │                   │
│ 142 auth tests     62%  │ analyzing failure...          │                   │
│ 143 session docs queued │ editing tests/auth.test.ts    │                   │
│ 144 review API    wait  │ running pnpm test...          │                   │
└─────────────────────────┴───────────────────────────────┴───────────────────┘
 router:on | verify:on | fallback:on | context:3.2k | merge:you
```

TUI nên có màn hình đọc nhanh:
```text
┌ Task contract ───────────────────────────────┐
│ Scope             Fix flaky auth test       │
│ Allowed           src/auth/** tests/auth/** │
│ Denied            .env secrets/** infra/**  │
│ Max changes       200 lines                 │
│ New dependencies  forbidden                 │
│ Reviewer          Codex                     │
│ Verification      required                  │
└─────────────────────────────────────────────┘
```

## Screens

### Overview
- system health;
- active tasks;
- recent route/fallback events;
- pending approvals.

### Departments
- department purpose;
- assigned agents;
- capability map;
- primary/fallback.

### Tasks
- queue;
- priority;
- state;
- current owner;
- progress.

### Execution
- real PTY;
- attach/input;
- pause/cancel/reroute;
- scrollback/search.

### Contract
- scope;
- files;
- constraints;
- success criteria;
- policies.

### Diff
- changed files;
- unified diff;
- summary;
- suspicious changes.

### Review
- findings;
- approve/request changes/reject.

### Verification
- checks;
- logs;
- evidence;
- policy decision.

### Agents
- readiness;
- capabilities;
- load;
- current task;
- heartbeat;
- degraded reason.

## Tabs
Task-centric:
`[TASK-142 auth] [TASK-143 docs] [+]`

Inside task:
`Overview | Exec | Contract | Diff | Review | Verify | Logs`

## Keybindings MVP
- `q`: detach TUI
- `?`: help
- `j/k`: move
- `enter`: open
- `tab`: next panel
- `1..7`: task views
- `r`: reroute
- `a`: approve
- `x`: reject/cancel
- `/`: search
- `:`: command palette
- `ctrl+p`: quick switcher

## Color semantics
- green: ready/pass/completed
- amber: busy/waiting/partial
- red: failed/blocked/security
- blue: selected/running
- gray: offline/unknown

## Responsive terminal
- Wide: 3 columns.
- Medium: left sidebar + main; right panel toggle.
- Narrow/SSH/mobile: one panel at a time with switcher.
