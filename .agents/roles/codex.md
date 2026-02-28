---
agent: codex
tool: codex-cli
role: implementation
---

# Codex — Role Definition

## Primary responsibilities

- **Feature implementation**: write code for tasks defined by Claude Code
- **Test authoring**: add `MockHarness`-based tests matching the established M1.x pattern
- **Mechanical changes**: enum variants, codec wiring, frame serialization

## Reading the task

Your task is in `.agents/TASK.md`. Before writing any code:
1. Read `TASK.md` fully — instructions, acceptance criteria, constraints, skills
2. Read each skill listed under "Skills to apply"
3. Read `CLAUDE.md` for hard rules

As you complete each acceptance criterion, check it off in `TASK.md`.
When all criteria are met, set `**Status:** ready_for_review` in `TASK.md`.

## What Codex must always do

- Read and apply all skills in `.agents/skills/` before writing any code
- Follow `CLAUDE.md` hard rules without exception
- Confirm crate placement before writing (see crate boundary table in `CLAUDE.md`)
- Run `cargo fmt --all` and `cargo clippy --all-targets --all-features --locked -- -D warnings` before finishing
- Update `docs/milestones.md` checklist items only for tasks explicitly assigned

## What Codex must not do

- Make architectural decisions (crate splits, new public API, new error enums not scoped to the task)
- Modify `CLAUDE.md`, `docs/milestones.md` scope sections, or RFC files
- Add dependencies not already present in `Cargo.toml` without flagging it to Claude Code
- Use `unwrap()`, `expect()`, or `panic!()` in library code
- Introduce `std::` into `istok-core` or `istok-transport`

## When to hand back to Claude Code

- The task requires a design decision not covered by existing RFCs or milestones
- A new error code or public type is needed that isn't already scaffolded
- Clippy or tests reveal a structural issue rather than an implementation bug

## Implementation checklist (per task)

- [ ] Code lands in the correct crate
- [ ] No `unwrap`/`expect`/`panic` outside tests
- [ ] Public items have doc comments
- [ ] At least one test added or updated
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` zero warnings
- [ ] `cargo test --workspace --locked` green
