# Agents — Istok

This directory defines the multi-agent setup for the Istok project.
Two agents collaborate with distinct responsibilities:

| Agent | Tool | Primary role |
|---|---|---|
| Claude Code | `claude` CLI | Design, architecture, planning, RFCs, code review |
| Codex | `codex` CLI | Feature implementation, mechanical code changes |

## Role division

**Claude Code** (`roles/claude.md`):
- Owns milestone scoping and definition-of-done criteria
- Authors RFCs before large refactors
- Reviews Codex output for correctness and architectural fit
- Makes decisions about crate boundaries and public API shape

**Codex** (`roles/codex.md`):
- Implements features as specified in milestones and RFCs
- Follows all skills in `skills/` unconditionally
- Does not make architectural decisions or modify `docs/milestones.md`
- Hands off to Claude Code when a task requires design judgment

## Skills

Skills in `skills/` apply to both agents unless the frontmatter says otherwise.
Each skill is a folder containing a `SKILL.md` with YAML frontmatter and Markdown body.

```
skills/
  rust_style/         — Rust coding conventions for all crates
  protocol_testing/   — Deterministic harness and codec test patterns
  no_std_rules/       — no_std compatibility requirements for istok-core
  milestone_playbook/ — How to scope, implement, and close out a milestone
```

## Task handoff workflow

Work is handed from Claude Code to Codex via `.agents/TASK.md` (gitignored, local only).

```
Claude Code writes TASK.md
  → user runs Codex: "read .agents/TASK.md and implement it"
    → Codex implements, checks off acceptance criteria, sets status to ready_for_review
      → user reviews code
        → user asks Claude Code to review
          → Claude Code approves or writes a new TASK.md with follow-up
```

`TASK.md` is overwritten for each new task. History lives in git commits, not the file.

## Reading order for agents

1. Read your own role file in `roles/`
2. Read all applicable skills in `skills/`
3. Cross-check against `CLAUDE.md` in the repo root (authoritative on hard rules)
4. **Codex only:** read `.agents/TASK.md` for the current task
