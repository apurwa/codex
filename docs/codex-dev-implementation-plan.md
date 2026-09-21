# Codex Dev Agents implementation plan

This plan turns the upstream/plugin research into an incremental maintenance
program. It protects the current product contract while reducing the cost of
future OpenAI Codex updates.

## Non-negotiable contract

- `codex` remains the official OpenAI binary.
- `codex-dev agents` is the only user-facing customized command.
- `integration/codex-dev` is the release branch.
- Feature and sync work stays in `.worktrees/`.
- Build and install only from a clean integration worktree.
- A source build is not considered released until the installed Ghostty PTY
  smoke test passes.

## Phase 1: inventory the downstream delta

**Goal:** establish facts before refactoring.

Record the merge-base with `upstream/main`, the commits unique to the
integration branch, changed source files, changed snapshots, and installed
binary receipt. Classify each change as upstream candidate, plugin/config
candidate, intentional downstream UX, or compatibility fix.

**Areas:** Git history, `codex-rs/tui`, snapshots, launcher/install script,
`docs/codex-dev-customization-manifest.md`.

**Prerequisites:** clean integration worktree or an explicit list of preserved
uncommitted changes; fetched `upstream/main`.

**Risk:** counting generated snapshots as independent product behavior.

**Acceptance:** a reviewed inventory with one owner and disposition for every
custom commit and every high-churn TUI area. Separate commit: `chore: audit
codex-dev downstream delta`.

## Phase 2: separate Layer 1 from Layer 2

**Goal:** move behavior that official Codex can already configure into the
official path and isolate unavoidable renderer changes.

Layer 1 includes plugins, skills, agents, commands, hooks, apps, MCP, and
supported configuration. Layer 2 includes transcript bands, labels, colors,
spacing, composer layout, agents-view presentation, and statusline rendering
that require Rust TUI changes.

**Risk:** accidentally changing the user-facing experience while reducing the
fork. Keep the current rendering snapshots as the Layer 2 contract.

**Acceptance:** the manifest names the owning layer and fallback behavior for
each customization. Separate documentation/cleanup commits; no installation.

## Phase 3: gate Layer 2 rendering

**Goal:** make upstream behavior and local UX independently testable.

Introduce a clearly named configuration or environment switch for the local
renderer layer. Default behavior must remain explicit and documented; do not
silently change the official `codex` binary. Keep separate snapshots for the
upstream and codex-dev paths.

**Areas:** TUI style/rendering modules, config schema if a config key is used,
snapshot tests, launcher environment.

**Risk:** a toggle that changes semantics rather than presentation, or a
config key accidentally loaded by official `codex`.

**Acceptance:** both paths compile, render, and have focused snapshots; the
toggle is available only to `codex-dev agents`. Separate feature commit.

## Phase 4: upstream stable extension seams

**Goal:** reduce future fork conflicts rather than asking upstream to adopt our
personal colors.

Prepare small upstream PRs for generic seams: theme/style providers,
transcript label formatting, statusline item styling, and renderer hooks. Keep
product-specific preferences downstream until a seam is accepted.

**Prerequisites:** Phase 1 inventory and a minimal reproduction without local
branding.

**Risk:** proposing an API before the rendering responsibilities are stable.

**Acceptance:** each proposal has a focused test and no dependency on the
`codex-dev` launcher. Separate upstream PRs, not a merge into our release
branch.

## Phase 5: version the launcher

**Goal:** make the customized command reproducible and auditable.

Put the `codex-dev` wrapper under repository control, install it alongside the
binary, and test that `codex` is untouched. The launcher must point to the
installed integration binary and expose only `codex-dev agents` as the
customized entry point.

**Areas:** `scripts/install-codex-dev.sh`, launcher source, install tests,
receipt format.

**Acceptance:** a clean machine can identify launcher version, source commit,
binary hash, and install time without relying on an untracked file.

## Phase 6: automate behavior gates

**Goal:** catch regressions before a Ghostty session.

Add focused TUI snapshots and a scripted PTY test for agent-row opening,
scrolling, selection/copy, both composers, project selection, session title,
statusline, and return-to-agents focus. Keep Ghostty as the final visual check,
not the only check.

**Risk:** terminal-dependent tests becoming flaky. Keep the scripted test small,
deterministic, and separate from visual acceptance.

**Acceptance:** the test suite fails when a required interaction regresses and
passes against the installed integration binary. Separate test commit.

## Phase 7: make provenance self-maintaining

**Goal:** eliminate hand-maintained release ambiguity.

Extend the install flow to record upstream SHA, integration SHA, launcher
version, binary hash, test result, and Ghostty smoke-test status in a machine-
readable receipt. Generate the human release ledger from those receipts or
update it from the same command.

**Acceptance:** every installed build has one provenance record and a rollback
target. Separate tooling/docs commit.

## Phase 8: choose the sync model

**Goal:** make upstream updates reviewable and repeatable.

After the delta inventory, prefer an ordered downstream patch stack rebased
onto `upstream/main` when commits are cleanly separable. Use a merge when
upstream history or shared integration commits make rebase riskier. In either
case, use an isolated sync worktree, enable `git rerere`, review every reused
resolution, run all gates, and merge only the reviewed result into
`integration/codex-dev`.

**Acceptance:** one documented sync command sequence, no feature-branch
installation, and a release receipt tied to the exact upstream base.

## Execution order

1. Complete Phase 1 and review the inventory.
2. Complete Phase 2 without changing runtime behavior.
3. Restore the Rust toolchain, build the current integration commit, install it,
   and run the original UX regression checklist.
4. Implement Phase 3 only after that baseline is captured.
5. Pursue upstream seams and launcher/test/provenance work in separate commits.

Do not combine an upstream sync, renderer refactor, launcher change, and
installation in one step.

## Initial inventory snapshot

Collected from `integration/codex-dev` on 2026-09-21 before further changes:

- Upstream merge-base: `e269f2164cbb9f499e4f22301c393500e2a831f3`.
- Integration commits ahead of `upstream/main`: 45.
- Changed paths relative to the merge-base: 367.
- TUI source paths in that delta: 355.
- Snapshot paths in that delta: 237.
- The worktree also contains an uncommitted change in
  `codex-rs/tui/src/app/agents_overview_view.rs`; it is preserved and must be
  reviewed before the next release commit.

This confirms that the renderer delta is currently broad. It is a reason to
measure and isolate the fork before attempting another upstream update, not a
reason to delete or reset existing work.
