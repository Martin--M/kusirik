---
trigger: always_on
---

# Changelog Maintenance Rules

Keep [docs/changelog.md](../../docs/changelog.md) continuously up to date as changes are implemented rather than reconstructing release notes retroactively.

## Guidelines
- Maintain a `## [Unreleased]` section at the top of `docs/changelog.md` above the latest released version, grouped into `### Features`, `### Performance & Architecture`, `### Fixes`, and `### Internal` (only include subsections that have entries).
- Whenever implementing a user-facing feature, bug fix, performance improvement, or notable architectural/tooling change, add a concise bullet entry under `## [Unreleased]` as part of the change.
- When cutting a release (e.g. `release: vX.Y.Z`), promote the contents of `## [Unreleased]` into the new `## vX.Y.Z` section and keep a fresh `## [Unreleased]` heading ready for subsequent work.
