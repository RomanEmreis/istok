---
name: Milestone Playbook
description: How to scope, implement, and close milestones — vertical slices, DoD checklists, scaffold rules, and sequencing constraints.
applies_to: [claude, codex]
triggers: [starting a milestone, scoping a task, closing a milestone, writing a DoD]
---

# Milestone Playbook

## Purpose

Defines how milestones are scoped, implemented, and closed. Keeps work vertical
and shippable at each step rather than building perfect subsystems in isolation.

## Milestone anatomy (Claude Code's responsibility)

Each milestone in `docs/milestones.md` must define:

```markdown
### Scope
What is included. What is explicitly excluded.

### Acceptance tests
Concrete list of tests that must pass (named, not vague).

### DoD checklist
- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Milestones.md updated
- [ ] No scaffold files removed without replacement

### no_std / min-deps notes
Any constraints this milestone adds or relaxes.
```

## Implementation approach (Codex's responsibility)

**Vertical slices, not horizontal layers.**

Good: "HEADERS frame written, DATA frame written, FIN on DATA — one end-to-end path works."
Bad: "Perfect frame encoder with all edge cases, but nothing calls it yet."

Order of work for a typical M1.x task:
1. Write the acceptance test first (it will fail).
2. Implement the minimum code to make it pass.
3. Check style, doc comments, no `unwrap`.
4. Run full test suite.
5. Tick the checklist item in `docs/milestones.md`.

## Scaffold files

Scaffold files (empty `lib.rs`, placeholder modules with a comment) are part of
the design — they mark where future code will live. Do not delete them unless
explicitly asked and a replacement is provided in the same PR.

## Milestone sequencing rules

- Do not start M(n+1) work until M(n) DoD checklist is fully checked off.
- `docs/milestones.md` is the canonical source of truth. If it says a task is open, it is open.
- Only Claude Code updates milestone scope or adds new tasks. Codex ticks completed items.

## Anti-patterns

- "While I'm here" refactors — do only what the milestone task specifies.
- Marking a task done in `milestones.md` before the acceptance test passes.
- Removing a scaffold file to "clean up" — it breaks the design intent.
- Implementing M2 features (QPACK, etc.) during an M1.x milestone.
