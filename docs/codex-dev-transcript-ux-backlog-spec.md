# Codex Dev transcript UX backlog

Status: review-ready specification for Claude and the integration owner.

## Goals

Improve transcript legibility without changing the upstream/default profile, mouse
selection, scrolling, composer behavior, or the established `codex-dev agents`
workflow. Every visual customization must remain behind `UiProfile::CodexDev`.

## Current visual contract

- `YOU` messages use the blue background and blue top/bottom rules.
- `CODEX · Tool Calls`, `CODEX · Update`, and `CODEX · Answer` use the neutral
  gray card/background treatment with maroon headings.
- Tool success markers are vivid green.
- Grouped tool calls have one intentional continuation spacer and a shared
  `CODEX · Tool Calls` heading; continuation rows retain the `┊` gutter.
- Upstream profile output must remain byte-identical and must not receive these
  customizations.

## Track 1 — ship now

### P0: grouped tool-call layout regression

Observed behavior: the first call appears immediately below `CODEX · Tool Calls`,
and continuation rows can look visually ungrouped or incorrectly indented.

Required behavior:

1. The group heading and every call use the same left gutter.
2. The first call has the same visual relationship to the heading as subsequent
   calls; use one deliberate spacer row, not an accidental double gap.
3. Continuation calls have exactly one vertical spacer and retain `┊` prefixes.
4. A grouped call must not create a second `CODEX · Tool Calls` heading.
5. Mouse selection and transcript scrolling must continue to work.

Implementation surface: `prepend_codex_tool_call_label`,
`ToolCallContinuationCell`, and `App::insert_history_cell`. Add focused direct
line-layout tests before changing snapshots.

This track is the release gate: reconcile CodexDev snapshots, run the full TUI
suite, build/install from integration, and perform the Ghostty smoke test.

## Track 2 — post-install backlog

### P1: tool-call metadata simplification

Review removing the default inline `0ms`, rendered line count, and repeated
`Ctrl+T details` suffix. The compact row should prioritize the action and result:

```text
┊ ✓ git status --short
```

Details remain available through an explicit expand action. Do not remove useful
failure output or make expansion inaccessible by keyboard/mouse. Compare ordinary
exec, MCP, web-search, patch, and background-terminal calls before implementation.

### P1: semantic activity labels

Use explicit labels where the activity is known: `Search`, `Web Search`, `Git`,
`Patch`, `MCP`, `Background Terminal`, and `Explored` only for exploratory
activity. A grouped heading may remain `CODEX · Tool Calls`; the child row should
communicate what actually happened.

### P1: result markers and hierarchy

Standardize success/failure/in-progress markers:

- vivid green `✓` for completed success;
- red `✗` for completed failure;
- neutral dot/bullet for pending or background work.

Document the meaning in the UI help and test each marker under light, dark,
ANSI16, and truecolor environments.

### P1: default expansion and code edits

Keep code edits collapsed by default in the transcript, with a clear expand
 affordance. Verify that expanded content preserves selection, scrolling, and
the original command/result ordering.

### P1: Agents-view project selector

Implement the project-directory selector in the new-task composer. It should show
the current directory, allow choosing a trusted project, preserve the selected
directory when creating a session, and remain usable from keyboard and mouse.
Keep the existing composer visible and bottom-sticky.

### P2: profile and upstream durability

- Add the optional `tui.ui_profile` config key after the launcher/env path is
  stable.
- Keep upstream-baseline tests for every gated surface.
- Reconcile legacy CodexDev snapshots with explicit
  `with_test_ui_profile(CodexDev, ...)`; never blanket-accept snapshots.
- Sync upstream only through an isolated worktree, review conflicts, run the
  baseline harness/full suite, then build/install from integration.

The Track 2 items must not block Track 1 installation. They should each receive
their own focused design review, tests, and isolated implementation commit.

## Validation gate

Before each install: focused layout tests, truecolor baseline harness, full TUI
suite, clean integration worktree, installed-binary provenance check, and manual
Ghostty smoke for opening/returning sessions, scrolling, mouse selection/copy,
Jump to bottom, resize, both composers, and the official `codex` command.
