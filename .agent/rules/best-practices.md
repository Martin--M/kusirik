---
trigger: always_on
---

# Engineering Best Practices & Industry Standards

Always prioritize software engineering best practices and established industry standards when implementing new code or fixing issues.

## Core Principles
- **Fix root causes, not symptoms**: Diagnose the underlying defect before editing; avoid fragile workarounds or silent fallbacks that mask real failures.
- **Single source of truth & DRY**: Centralize shared parsing, formatting, validation, and data-conversion helpers rather than duplicating ad-hoc logic across files.
- **Strict boundary contracts**: Keep schemas, Rust DTOs, Tauri IPC payloads, and TypeScript types strictly aligned (e.g., integer epoch units, nullability, and error shapes).
- **Explicit error handling**: Never swallow errors silently. Surface actionable, user-visible feedback (such as error states or toasts) and log structured diagnostics when operations fail.
- **Clean, modular, idiomatic code**: Follow idiomatic Rust and TypeScript/Vue 3 conventions, keep functions focused and readable, and avoid blocking UI/main threads with I/O or subprocess waits.
- **Verify with automated checks**: Add or update targeted unit tests for bug fixes and new behavior, and run all relevant verification checks before considering work complete:
  - Rust (`src-tauri/`): `cargo clippy --all-targets -- -D warnings` and `cargo test`
  - Frontend: `npm test` and `vue-tsc --noEmit`
