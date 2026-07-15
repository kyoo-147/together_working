# 00 - Product Brief

## Product Name

Together

## One-Line Description

Together is a local-first CLI/TUI operator console that lets developers keep working in Codex while observing, routing, verifying, and reviewing the AI worker activity behind the scenes.

## Current Product Direction

Together has moved beyond a Python skill/report prototype. The current product surface is a terminal-native app:

- `together` starts or connects to the local daemon.
- The TUI shows Project, Tasks, Task Monitor, Live Work Feed, Agents, Needs Attention, Settings, and Chat Dock.
- Codex app / skill requests and Together chat requests flow through the same daemon protocol.
- The daemon owns state, routing, proposals, review, verification, settings, and worker lifecycle.
- The TUI is a thin client that renders events and sends commands.

The near-term product is not "another chat app." Codex remains the primary conversation and integration surface. Together is the monitor, control plane, and evidence layer around that work.

## Problem

Developers increasingly use several coding agents across Codex, Claude, Gemini, OpenCode, Amp, and local CLIs. The work becomes hard to operate when:

- context grows too large;
- worker readiness is unclear;
- task scope is implicit;
- file permissions are not enforced;
- review and verification happen too late;
- provider failures stop the workflow;
- agent output lacks durable evidence;
- it is hard to know which worker changed which files.

## Solution

Together makes AI work observable and governed:

1. discover installed agents;
2. probe readiness and mark degraded workers without crashing the flow;
3. create scoped task contracts;
4. route work through deterministic scoring;
5. spawn real workers in PTYs;
6. stream worker output into the TUI;
7. accept chat requests from Codex skill/app and the Together chat dock;
8. create proposals before mutating state;
9. verify diff scope and denied-file rules;
10. block approval when verification fails;
11. record local events, settings, and runtime artifacts.

## Target Users

- developers using Codex plus one or more local AI coding CLIs;
- teams dogfooding multi-agent development workflows;
- local-first engineering teams that need auditability and control;
- builders comparing single-agent versus governed multi-worker workflows.

## Positioning

Together is:

- a Codex operator console;
- a local AI worker control plane;
- a task contract and verification layer;
- a routing and fallback layer;
- a measurement loop for real developer sessions.

Together is not:

- a Codex replacement;
- a general terminal multiplexer;
- a fully autonomous enterprise scheduler;
- a finished RBAC/audit/compliance platform.

## North-Star Experience

The user runs:

```powershell
together
```

and sees:

- which project and source are active;
- which tasks are queued, running, blocked, or awaiting review;
- which agents are ready, running, degraded, or offline;
- why a worker was selected;
- live PTY output from the worker;
- what needs attention now;
- whether verification passed or blocked approval;
- a chat dock for asking Codex/Together to create or inspect work.

## Current Implementation Status

The product currently includes:

- daemon auto-start/connect path;
- Monitor + Chat Dock TUI;
- CLI commands for chat, proposals, status, settings, doctor, and review actions;
- skill bridge helper for Codex-sourced chat submission;
- deterministic and assisted proposal parser shape;
- review and approval gate state;
- settings/theme presets with custom colors;
- responsive terminal render paths;
- Windows-first build, install, and package scripts;
- release checks and bridge tests.

## Active Dogfooding Questions

The current development loop should measure and improve:

- how quickly users understand the TUI;
- whether chat-first task creation is simpler than forms;
- whether Needs Attention highlights the right interventions;
- how often Codex is degraded and which fallback works best;
- how much routing visibility is useful without clutter;
- whether scope verification blocks the right failures;
- how much time Together adds or saves compared with Codex-only work.

## Roadmap Shape

1. Local CLI/TUI operator console.
2. Dogfooding and benchmark measurement loop.
3. Team control plane with shared policy templates.
4. CI, PR, and review integrations.
5. Enterprise AI workforce platform with RBAC, audit, budgets, sandboxing, and adaptive routing.
