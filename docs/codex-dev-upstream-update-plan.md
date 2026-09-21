# Codex Dev Upstream Update Plan

This is the canonical playbook for keeping our customized Codex CLI current
without losing UX changes or destabilizing the installed `codex-dev` command.

The companion [`codex-dev-customization-manifest.md`](codex-dev-customization-manifest.md)
is the source of truth for which behavior is intentionally downstream and
which behavior should move to upstream configuration or plugins.

## Command and branch contract

- Keep the official OpenAI CLI available as `codex`.
- Use `codex-dev agents` as the only user-facing command for our customized CLI.
- Treat `integration/codex-dev` as the releasable branch.
- Keep worktrees under `Codex CLI/.worktrees/`.
- Never build or install a user-facing binary directly from a feature or sync
  branch. Merge the reviewed result into `integration/codex-dev` first.

## Baseline recorded on 2026-09-15

| Item | Recorded value |
| --- | --- |
| Installed integration commit | `b50c7d1360` |
| Last identifiable OpenAI main-line base | `a505c71490` |
| Current OpenAI `main` | `a8964cb1bad67bc26a826fb07d1bef99c6a3f008` |
| Commits on OpenAI `main` since our base | 68 |
| Latest stable release | `rust-v0.154.0` / Codex CLI 0.154.0 |
| Stable release publication date | 2026-09-09 |

An update candidate exists. Our customization started from the official
main-line history, so the integration target is `upstream/main`. The latest
stable release is recorded for product/version awareness; do not merge the
stable tag into `integration/codex-dev` because its release history currently
diverges from the main-line base used by our fork.

## One-time remote setup

From the canonical repository:

```sh
git remote add upstream https://github.com/openai/codex.git
git fetch upstream main --tags
```

If `upstream` already exists, verify it with `git remote -v`, then fetch it.

Enable Git's recorded conflict-resolution support once on the development
machine. It reduces repeated manual work during rebases, but every reused
resolution still requires a diff review and tests:

```sh
git config rerere.enabled true
```

## Routine update check

Run this weekly, before starting a major customization, and promptly after an
OpenAI security release:

```sh
gh release view --repo openai/codex --json tagName,name,publishedAt,url
git fetch upstream main --tags
git log --oneline a505c71490..upstream/main
git rev-list --count a505c71490..upstream/main
```

After every completed sync, replace `a505c71490` in the comparison with the
new upstream commit recorded in the release ledger below. A detected update is
only a review candidate; it must not trigger an automatic merge or install.

## Controlled sync procedure

1. Confirm `integration/codex-dev` is clean and record its commit and installed
   build receipt.
2. Review the upstream release notes and commit list. Flag changes touching the
   TUI, session history, mouse handling, composer, status line, agents view,
   worktrees, configuration, authentication, or sandboxing.
3. Create an isolated worktree from the integration branch:

   ```sh
   git worktree add .worktrees/upstream-sync-YYYYMMDD \
     -b chore/upstream-sync-YYYYMMDD integration/codex-dev
   ```

4. In that worktree, merge `upstream/main` without committing immediately:

   ```sh
   git merge --no-commit upstream/main
   ```

5. Resolve conflicts by preserving upstream security and architectural changes,
   then deliberately reapplying our UX behavior. Do not accept either side in
   bulk for TUI files.
6. Audit our required behaviors before considering the sync complete:
   - agents rows open with keyboard and mouse;
   - selected agent row uses `#005f87` with white text;
   - transcript scroll, text selection, and jump-to-bottom all work;
   - user turns use a full-width band with `YOU`, top and bottom borders;
   - assistant turns use a legible `CODEX` heading;
   - session composer is bottom-sticky, bordered, and shows the bold session
     title;
   - agents composer is present and bottom-sticky;
   - customized bold/icon status line renders usage data honestly;
   - the official `codex` command remains untouched.
7. Run proportionate verification:

   ```sh
   git diff --check
   just test -p codex-tui --lib
   just fix -p codex-tui
   just fmt
   cargo build -p codex-cli --bin codex
   ```

8. Practice `codex-dev agents` in a real Ghostty PTY. Test opening a session,
   scrolling, selecting text, returning to agents, clicking rows, resizing, and
   using both composers.
9. Commit the sync branch. Merge it into `integration/codex-dev`, then build and
   install only from the integration worktree using the repository's
   `scripts/install-codex-dev.sh` flow.
10. Repeat the Ghostty smoke test against the installed binary. Push the
    integration branch only after it passes.

## Release and rollback policy

- Tag known-good builds with a monotonically increasing custom version, for
  example `codex-dev-v1.1.0-upstream-a8964cb1`.
- Never move an existing stable tag.
- Keep the prior installed binary until the new build passes the installed PTY
  smoke test.
- If validation fails, restore the previous known-good binary and leave the
  sync branch unmerged for diagnosis.

## Release ledger

Add one row for every installed custom release.

| Date | Custom tag | Integration commit | Upstream commit | Official stable observed | Tests | Ghostty smoke test |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-09-15 | pending | `b50c7d1360` | `a505c71490` base | `rust-v0.154.0` | Existing baseline | Existing baseline |

## Decision for the current update

Do not update the installed binary merely because 68 upstream commits are
available. First create the isolated sync worktree and audit the upstream
managed-worktree and TUI changes, since those areas can overlap our agents and
worktree customizations. Promote the result only after the complete verification
and installation gates above pass.
