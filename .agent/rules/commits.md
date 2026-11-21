---
trigger: always_on
---

# Git Commit Rules

Follow Linux kernel commit style with an emphasis on explaining **why** a change is made, and keep git history easy to review.

## Commit Message Format
- **Subject line**: Use lowercase subsystem/area prefixes followed by a concise imperative summary, with no trailing period (aim for <= 72 characters):
  - Examples: `tauri: db: ...`, `tauri: history: ...`, `frontend: ui: ...`, `frontend: views: movies: ...`, `repo: agent: ...`
- **Body** (wrap lines at ~72 characters):
  1. Start with the **why**: describe the current problem, limitation, bug, or motivation.
  2. Explain **what** changed and why this approach was chosen, written in imperative mood.
  3. Note the **impact** or trade-offs when helpful (e.g., performance, behavior, or regression coverage).
  4. Optional concise bullet list at the bottom summarizing touched components for multi-file commits.

## Commit Size & Scoping
- **Self-contained**: Every commit must be self-contained whenever possible—it should compile cleanly and leave tests passing on its own without depending on a follow-up commit to fix breakage.
- **Small and reviewable**: Keep each commit focused on a single logical change so diffs can be reviewed thoroughly and bisected cleanly.
- **Split large changes**: Whenever a task spans multiple distinct concerns (such as backend/schema changes vs. frontend UI updates, refactors vs. bug fixes, or tooling/config updates vs. application code), split it into smaller, ordered commits rather than one large commit.
