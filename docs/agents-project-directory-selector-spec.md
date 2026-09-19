# Agents View Project Directory Selector

## Goal

Make creating a session in another project directory obvious from the agents-view
composer. The directory should be selected before the task is submitted, and the
choice should apply only to the new task being created.

## Current problem

The agents view inherits its new-task directory from the command that launched it
(`codex-dev agents --cd <directory>`). The composer does not show that target or
provide a way to change it, so users must leave the view and restart the command.

## Proposed experience

The sticky agents-view composer gains a directory target row:

```text
New task                                      ~/Projects/Resume
──────────────────────────────────────────────────────────────
› Describe a new task
──────────────────────────────────────────────────────────────
↑↓ navigate  enter create  d directory  esc cancel
```

The target defaults to the directory supplied by `--cd`, or the current working
directory when no explicit directory was supplied.

Pressing `d` while the dashboard composer is focused opens a directory picker.
The picker offers:

- the current target;
- recently used project directories from the current agents-view process;
- project directories already represented in the agents list;
- a path input for entering or pasting an absolute path.

The picker validates that the path exists and is a directory. Invalid paths stay
in the picker with an inline error and do not change the current target.

Mouse clicking the target row opens the same picker. `Esc` cancels without changing
the target. Selecting a directory returns focus to the task composer.

## Submission semantics

- `Enter` creates the task in the selected directory.
- The selected directory is passed through the existing new-session request path.
- Existing sessions are never moved or modified.
- Changing the directory does not change the process working directory.
- Creating a worktree continues to use the selected directory as its parent.
- After successful submission, the selected directory becomes the default for the
  next new task during this agents-view process.

## Keyboard and accessibility

- `d`: open directory picker from the new-task composer.
- `↑/↓`: navigate picker entries.
- `Enter`: choose the highlighted directory.
- `Esc`: cancel picker or return to the task composer.
- The active directory is visible in the composer and in the picker title.
- Long paths use front truncation while preserving the final directory name.

## Scope boundaries

This change does not alter the normal conversation composer, existing session
working directories, the official `codex` command, or the `codex-dev agents`
command contract. The command-line `--cd` option remains supported.

## Verification plan

Add tests for:

1. default target from `--cd` / current directory;
2. picker open, selection, cancel, and invalid-path behavior;
3. mouse activation of the target row;
4. task submission using the selected directory;
5. worktree creation using the selected directory;
6. sticky composer placement and narrow-terminal truncation;
7. session list focus returning after picker cancellation/submission.

## Follow-up transcript polish

After this selector ships, consider using forest green `#0B6623` for successful
tool-call ticks and adding one blank visual row between separate `CODEX · Tool
Calls` groups. These are intentionally separate from directory selection so the
two changes can be tested and reviewed independently.

Manual Ghostty smoke test:

```sh
codex-dev agents --cd "$HOME/Projects/Resume"
```

Open the directory picker, choose another project, submit a task, and verify that
the new session appears under the selected project while existing sessions remain
unchanged.
