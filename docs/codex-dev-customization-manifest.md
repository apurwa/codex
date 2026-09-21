# Codex Dev Customization Manifest

This file is the boundary between upstream Codex and the local `codex-dev`
product. It is intentionally small and reviewable. A customization belongs
here only when it changes the user-facing `codex-dev agents` experience and is
not already provided by upstream configuration or the official plugin system.

## Current baseline

- Canonical source: `upstream/main` from `https://github.com/openai/codex`
- Release branch: `integration/codex-dev`
- User-facing command: `codex-dev agents`
- Official command: `codex` (must remain unchanged)
- Build/install source: this integration worktree only

Record the exact upstream base and installed integration commit in the release
ledger in [`codex-dev-upstream-update-plan.md`](codex-dev-upstream-update-plan.md)
for every promoted build.

## Customization inventory

| Area | Local behavior | Primary implementation | Upstream strategy |
| --- | --- | --- | --- |
| Agents overview | Project directory selection, resilient handling of missing rollouts, and session attachment behavior | `codex-rs/tui/src/app/agents_overview.rs`, `codex-rs/tui/src/app/event_dispatch.rs` | Keep as a focused TUI patch; re-audit when agents/session APIs change |
| Transcript | Full-width user bands, `YOU` labels, `CODEX` headings, grouped tool-call presentation, spacing, colors, and selectable/scrollable output | `codex-rs/tui/src/history_cell/`, `codex-rs/tui/src/chatwidget/`, `codex-rs/tui/src/style.rs` | Keep renderer changes isolated; prefer upstream configuration when equivalent exists |
| Composer | Sticky session and agents composers, session title, borders, and project selector | `codex-rs/tui/src/bottom_pane/`, `codex-rs/tui/src/chatwidget/`, agents overview modules | Reconcile manually with upstream layout changes; protect with snapshots and PTY checks |
| Status line | Custom multi-line status presentation and accent color | `codex-rs/tui/src/status_line_style.rs` and related TUI modules | Prefer upstream status-line configuration; retain only presentation differences unavailable upstream |

## Extension boundary

Codex plugins are appropriate for skills, agents, commands, hooks, apps, and
MCP integrations. They are not currently a drop-in replacement for changing
the internal Rust TUI renderers listed above. Therefore, this project uses a
hybrid model:

1. use official config/plugins whenever they expose the needed behavior;
2. keep unavoidable visual/TUI changes as small downstream commits;
3. submit generally useful behavior upstream so the local patch can eventually
   be deleted.

## Review rules for an upstream update

Before merging upstream, classify every local commit as one of:

- **Upstream candidate**: behavior that should be proposed to OpenAI Codex.
- **Config/plugin candidate**: behavior that can move out of the fork.
- **Downstream UX**: an intentional local presentation choice.
- **Compatibility fix**: code required because upstream changed an interface.

Do not flatten all local changes into one merge commit. Preserve these
categories in separate commits so conflicts are easy to inspect and revert.

## Required gates

An upstream update is not releasable until all of these pass:

1. `git diff --check` and formatting.
2. TUI unit/snapshot tests covering transcript, agents, composer, and status.
3. A clean `cargo build -p codex-cli --bin codex` from the integration worktree.
4. A real Ghostty PTY smoke test of mouse opening, scrolling, text selection,
   both composers, project selection, and returning to the agents view.
5. Verification that `codex` still resolves to the official installation and
   only `codex-dev agents` resolves to the customized binary.

If any gate fails, keep the sync worktree for diagnosis and do not replace the
installed binary.
