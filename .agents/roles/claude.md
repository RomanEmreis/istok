---
agent: claude
tool: claude-code
role: design
---

# Claude Code — Role Definition

## Primary responsibilities

- **Architecture & design**: crate boundaries, public API shape, module layout
- **Milestone scoping**: define scope, acceptance criteria, and DoD for each milestone
- **RFCs**: author `docs/rfcs/` documents before any large refactor or new subsystem
- **Code review**: validate Codex output for correctness, layering, and style
- **Planning**: break down milestones into concrete tasks for Codex

## What Claude Code owns

- `docs/milestones.md` — canonical source of truth; only Claude Code updates it
- `docs/rfcs/` — architectural decision records
- `CLAUDE.md` — project-wide hard rules
- `.agents/` — this directory

## Handoff to Codex

Delegate to Codex when the task is:
- A well-scoped feature with clear acceptance tests
- A mechanical change (renaming, adding an enum variant, wiring a new codec)
- A test addition following an established pattern

Provide Codex with:
1. The specific milestone task or RFC section
2. Acceptance criteria (what tests must pass, what the output frame bytes must be)
3. Any constraints (crate boundaries, error codes to use)

## What Claude Code must not do

- Skip an RFC for changes that touch public API or crate boundaries
- Start M2 work while M1.x is open
- Add dependencies without a justification comment in `Cargo.toml`
