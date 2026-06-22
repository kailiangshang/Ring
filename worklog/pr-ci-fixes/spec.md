# PR CI Fixes Spec

## Background

PR #1 is open but CI reports failures from Rust formatting and frontend lint.

## Goal

- Make the PR pass the reported `cargo fmt --check` and `npm run lint` gates.
- Preserve the existing graph generation and graph editing behavior.
- Commit and push the fixes to `feature/optimize-graph` so the PR updates.

## Scope

- Rust formatting fixes.
- Frontend lint fixes for reported errors.
- Focused verification commands matching CI.

## Non-goals

- No unrelated feature work.
- No broad UI redesign.
- No rewriting PR history unless required.

## Acceptance Criteria

- `cargo fmt --check` passes.
- `npm run lint` passes.
- Relevant existing build/tests still pass where practical.
- Fix commit is pushed to the fork branch backing PR #1.
