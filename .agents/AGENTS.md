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

## Reading order for agents

1. Read your own role file in `roles/`
2. Read all applicable skills in `skills/`
3. Cross-check against `CLAUDE.md` in the repo root (authoritative on hard rules)
