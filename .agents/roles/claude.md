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

To hand off, write `.agents/TASK.md` using this structure:

```markdown
# Task — <Milestone>: <short title>

**Status:** pending

## Instructions
<concrete what-to-do, no ambiguity>

## Files to touch
- `crates/...`

## Acceptance criteria
- [ ] <specific test or observable outcome>
- [ ] Clippy clean (`cargo clippy --all-targets --all-features --locked -- -D warnings`)
- [ ] All tests pass (`cargo test --workspace --locked`)

## Constraints
- <what not to touch, which error codes to use, etc.>

## Skills to apply
- rust_style, protocol_testing  ← list applicable skills from .agents/skills/
```

Tell the user: "TASK.md is ready — pass it to Codex."
After Codex finishes and the user asks for review, read the diff and TASK.md together.

## What Claude Code must not do

- Skip an RFC for changes that touch public API or crate boundaries
- Start M2 work while M1.x is open
- Add dependencies without a justification comment in `Cargo.toml`
