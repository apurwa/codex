# Codex-dev concurrent session coordination

This protocol lets multiple Codex sessions work at the same time without
editing the same worktree or silently overwriting one another's changes.

## Ownership model

- Every feature session gets its own worktree and branch.
- `.worktrees/integration` is only for integration, validation, and install.
- Only the session holding the integration lease may merge into or install from
  the integration worktree.
- Coordination state lives under the shared Git directory at
  `codex-dev-coordination/`, so it is visible from every worktree and never
  becomes a merge conflict.

Create a feature worktree from the current integration tip:

```sh
git worktree add -b feat/<name> \
  ".worktrees/<name>" integration/codex-dev
```

## Claiming files

Before editing, claim the smallest useful path set:

```sh
scripts/codex-dev-session.sh claim ui-status \
  tui/src/status_indicator_widget.rs tui/src/summary_shimmer.rs
```

Claims are exclusive. A claim for `tui/src/style.rs` conflicts with a claim
for that file or any child path. This is a coordination guard, not a Git
replacement: the session still commits its work and leaves its worktree clean
before handoff.

Inspect active work:

```sh
scripts/codex-dev-session.sh status
```

## Handoff and integration

Finish with a commit and validation result. The command writes a durable
handoff under the shared coordination directory and archives the active claim:

```sh
scripts/codex-dev-session.sh finish ui-status <commit> \
  'cargo build --locked -p codex-cli --bin codex passed' \
  'Preserves shimmer; changes only active status color.'
```

The integration session acquires the integration lease before merging,
validating, or installing:

```sh
scripts/codex-dev-session.sh integration acquire integrator
git -C .worktrees/integration cherry-pick <commit>
# build, smoke-test, and install from the clean integration worktree
scripts/codex-dev-session.sh integration release integrator
```

If a feature session needs to touch a file already claimed, it records a
handoff and waits for the owner to finish. Parallel work continues on all
other unclaimed paths; no one needs to pause the entire project.

## Rules for Codex sessions

1. Run `status` before starting.
2. Create a worktree and claim files before editing.
3. Never edit another session's worktree.
4. Never install `codex-dev` from a feature worktree.
5. Finish with a commit, validation result, and handoff.
6. Keep the integration lease only for the merge/build/smoke/install window.

The official `codex` command remains outside this workflow and must not be
modified.
