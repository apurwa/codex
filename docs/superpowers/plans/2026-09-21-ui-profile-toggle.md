# UiProfile Runtime Toggle — Implementation Plan

> **For agentic workers:** implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Each task ends with an independently testable deliverable and its own commit.

**Goal:** Gate every `codex-dev` visual customization behind a runtime `UiProfile` toggle so that the default (`Upstream`) render path is byte-identical to upstream Codex, and the `codex-dev` identity renders only when the launcher opts in.

**Architecture:** A process-global `UiProfile { Upstream, CodexDev }`, resolved once at TUI startup from the `CODEX_UI_PROFILE` env var (and, later, a `tui.ui_profile` config key), mirroring the existing `terminal_palette` startup-global pattern. The free styling helpers in `style.rs` read the global directly; a handful of inline call sites that bypass `style.rs` each gate on it. Snapshot tests assert both renders: `Upstream` == upstream bytes, `CodexDev` == the current codex-dev look.

**Tech Stack:** Rust (pinned 1.95.0), ratatui/crossterm TUI, `insta` + `TestBackend` snapshots.

**Spec:** `docs/codex-dev-master-spec.md` §6 (Layer 2 Spec) and §12 step 2. This plan is the Phase-B execution detail.

## Global Constraints

- Official `codex` stays upstream-identical; only `codex-dev agents` is user-facing.
- Never touch `CODEX_SANDBOX*`.
- Default `UiProfile` is `Upstream`. The `codex-dev` launcher exports `CODEX_UI_PROFILE=codex-dev`.
- **Gating invariant:** with `UiProfile::Upstream`, rendered output (the `TestBackend` buffer `Debug`, which encodes fg/bg/modifiers) is byte-identical to `upstream/main`. Obtain each upstream baseline with `git show upstream/main:<path>`.
- Rendering code reads the profile ONLY through `crate::ui_profile::ui_profile()` (so the `#[cfg(test)]` thread-local override applies). Never read the env var or global directly at a render site.
- Never edit an upstream `.snap` in place to make it pass; add a new codex-dev snapshot module instead (`#[path = "..._tests.rs"]`).
- Keep each task's diff under ~800 lines; stage larger areas.
- Read the profile via a free function, not a bool parameter (AGENTS.md API rules). Inline `format!` args; collapse `if`s; exhaustive `match` on `UiProfile`.
- After code changes: `just fmt`; per-crate `just test -p codex-tui`; `just fix -p codex-tui` before finalizing. Do not run the full suite without asking.

---

### Task 1: UiProfile infrastructure (COMPLETE — commit on `feat/ui-profile-toggle`)

**Status:** Implemented in this branch. Render-neutral: establishes the switch, changes no pixels.

**Files:**
- Create: `codex-rs/tui/src/ui_profile.rs` — `enum UiProfile { Upstream (default), CodexDev }`, `CODEX_UI_PROFILE_ENV_VAR`, `set_ui_profile_from_env()` (freezes a `OnceLock`), `ui_profile()` getter, `#[cfg(test)] with_test_ui_profile()` thread-local override.
- Create: `codex-rs/tui/src/ui_profile_tests.rs` — env-value mapping, default-is-upstream, scoped override + restore.
- Modify: `codex-rs/tui/src/lib.rs` — `mod ui_profile;`.
- Modify: `codex-rs/tui/src/tui.rs` (`init()`, after `set_panic_hook()`) — call `set_ui_profile_from_env()` + `tracing::debug!` the resolved profile.
- Modify: `scripts/codex-dev` — `export CODEX_UI_PROFILE=codex-dev` before `exec`.

**Interfaces produced (later tasks consume these):**
- `crate::ui_profile::ui_profile() -> UiProfile`
- `crate::ui_profile::UiProfile::{Upstream, CodexDev}`
- `#[cfg(test)] crate::ui_profile::with_test_ui_profile(profile, || …)`

- [x] Enum + global + env read + startup wiring + launcher opt-in + unit tests, `just test -p codex-tui` green.

---

## Gating pattern (applies to Tasks 2–6)

For each customization site, obtain the upstream baseline and branch on the profile. Central helpers (Task 2) return the right `Style`; structural sites (Tasks 3–6) choose whole branches.

```rust
// At a style.rs helper:
pub(crate) fn codex_label_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => Style::default().fg(Color::Rgb(128, 0, 0)).bold(),
        crate::ui_profile::UiProfile::Upstream => /* the exact upstream style for this call site */,
    }
}
```

```rust
// At a structural site that upstream does not have at all (e.g. the YOU band):
if crate::ui_profile::ui_profile() == crate::ui_profile::UiProfile::CodexDev {
    // push the codex-dev band/label/rail
}
// else: render exactly the upstream lines
```

**Test pattern (each task adds this for its area):**

```rust
#[test]
fn <area>_upstream_profile_matches_upstream() {
    let buf = with_test_ui_profile(UiProfile::Upstream, || render_<area>_to_buffer());
    insta::assert_snapshot!("<area>_upstream", format!("{buf:?}"));
}

#[test]
fn <area>_codex_dev_profile_renders_customization() {
    let buf = with_test_ui_profile(UiProfile::CodexDev, || render_<area>_to_buffer());
    insta::assert_snapshot!("<area>_codex_dev", format!("{buf:?}"));
}
```

The `<area>_upstream` snapshot must equal the render produced by a clean `upstream/main` build of the same widget (diff against `git show upstream/main:<file>` output; when in doubt, render the same fixture on a scratch upstream checkout and copy its buffer).

---

### Task 2: Gate the central `style.rs` helpers

Covers customization groups A/C/D/I via the shared helpers, so every call site inherits the gate.

**Files:**
- Modify: `codex-rs/tui/src/style.rs:16-39` — `codex_label_style()` (maroon `Rgb(128,0,0).bold()`), `tool_success_style()` (`Rgb(0x00,0xc8,0x53).bold()`) + `success_marker()`, `selected_control_style()` (white-on-blue `Rgb(0,95,135)`), and `transcript_user_message_style()`/`transcript_user_message_bg_rgb` (`style.rs:76-82`).
- Baseline source: `git show upstream/main:codex-rs/tui/src/style.rs`. `codex_label_style`/`tool_success_style`/`success_marker` are absent upstream — under `Upstream`, return what upstream used at each *call site* (Task 3 removes the CODEX label entirely upstream, so `codex_label_style`'s `Upstream` arm only matters where upstream also styled a label; if no upstream caller exists, the helper is only reached under `CodexDev`). `selected_control_style` and `accent_style` exist upstream — copy their upstream bodies verbatim into the `Upstream` arm.
- Test: `codex-rs/tui/src/style_tests.rs` (new `#[path]` module) — assert each helper's `Upstream` arm equals the upstream `Style` and its `CodexDev` arm equals the current value, using `with_test_ui_profile`.

- [ ] Step 1: Write failing tests asserting `ui_profile()`-dependent return values for each helper (both arms).
- [ ] Step 2: `just test -p codex-tui` — expect FAIL (helpers ignore the profile).
- [ ] Step 3: Branch each helper on `ui_profile()`; `CodexDev` = current literals, `Upstream` = upstream body.
- [ ] Step 4: `just test -p codex-tui` — expect PASS.
- [ ] Step 5: `just fix -p codex-tui`, `just fmt`, commit.

---

### Task 3: Gate the YOU / CODEX band structure

The bands, labels, rails, and rules are downstream-*added lines*, not just restyled — so gate the insertion, not only the color.

**Files:**
- Modify: `codex-rs/tui/src/history_cell/messages.rs:264-305` (`UserHistoryCell::display_hyperlink_lines`: top rule 266-269, `"YOU"` label 270, left rail 288-292, bottom rule 297) and `messages.rs:308-310` (`background_style`), plus `messages.rs:438-441` and `:620-622` (`"CODEX"` label via `codex_label_style()`).
- Modify: `codex-rs/tui/src/history_cell/mod.rs:150-176` (`prepend_codex_tool_call_label` / `prepend_codex_tool_call_hyperlink_label` → `"CODEX · Tool Calls"`).
- Modify: `codex-rs/tui/src/history_cell/approvals.rs:24-53` (same YOU band for approvals).
- Baseline: `git show upstream/main:codex-rs/tui/src/history_cell/messages.rs` (and `mod.rs`, `approvals.rs`) — render the upstream user/assistant/approval cell to confirm the exact lines the `Upstream` arm must produce.
- Test: extend `codex-rs/tui/src/history_cell/messages_tests.rs` (pattern at `:143` `user_messages_have_a_visible_label_and_rail…`, snapshot at `:192`). Add `_upstream` and `_codex_dev` variants; the existing snapshot becomes the `_codex_dev` one.

- [ ] Steps: failing dual-profile snapshot test → gate the band/label/rail/rule insertion on `CodexDev` → upstream arm reproduces upstream lines → PASS → fix/fmt/commit. Keep messages.rs edits minimal (it is a high-touch file; do not add unrelated helpers).

---

### Task 4: Gate composer borders, session title, agents composer

**Files:**
- Modify: `codex-rs/tui/src/bottom_pane/chat_composer.rs:4958-4966` — blue `Borders::TOP|BOTTOM` + `.border_style(Rgb(0,95,135))`. Under `Upstream`, use the upstream block (borders/style from `git show upstream/main:codex-rs/tui/src/bottom_pane/chat_composer.rs`).
- Modify: `codex-rs/tui/src/bottom_pane/mod.rs:2200-2202` — blue bold right-aligned session title. Gate the title line (upstream likely omits it or styles it plainly).
- Modify: `codex-rs/tui/src/app/agents_overview_composer.rs:23` — `borders_enabled: true`. Under `Upstream`, use upstream's value; the agents overview is downstream-only, so this arm mainly matters for the shared composer's byte-identity.
- Test: extend `codex-rs/tui/src/bottom_pane/chat_composer.rs` snapshot tests (pattern at `:5197` `light_terminal_palette_renders_light_composer_snapshot`, snapshot at `:5215`) with dual-profile variants.

- [ ] Steps: failing dual-profile composer snapshot → gate border/title/agents-composer on profile → upstream arm == upstream → PASS → fix/fmt/commit.

---

### Task 5: Gate the unified status-line accent

**Files:**
- Modify: `codex-rs/tui/src/bottom_pane/status_line_style.rs:17` (`STATUS_LINE_PRIMARY_BLUE = Rgb(0,95,135)`) and `:118-139` (downstream renders every non-thread item in that blue+bold). Under `Upstream`, restore upstream's per-item theme-scope accents (`fallback_style()` cyan/green/magenta) — copy from `git show upstream/main:codex-rs/tui/src/bottom_pane/status_line_style.rs`.
- Test: `status_line_style.rs:217-348` unit tests currently assert `fg == STATUS_LINE_PRIMARY_BLUE` and `!DIM`. Split into `Upstream`/`CodexDev` variants via `with_test_ui_profile`; the blue+bold assertions move under the `CodexDev` arm; add `Upstream` assertions for the upstream per-item accents.

- [ ] Steps: adapt the existing unit tests to both profiles (failing) → gate the accent block on `ui_profile()` → upstream arm == upstream per-item accents → PASS → fix/fmt/commit.

---

### Task 6: Gate Nerd-Font icons + bold status rows

**Files:**
- Modify: `codex-rs/tui/src/chatwidget/status_surfaces.rs:229-271` (`refresh_status_line_from_selections`) — Nerd-Font glyphs (`\u{f07b}`, `\u{f09b}`, `\u{f017}`, `\u{f2db}`), forced `.bold().remove_modifier(DIM)`, icon prefix span. This site has `self.local_settings` in scope, so it MAY read `self.local_settings.tui.ui_profile` once Task 7 lands the config field; until then, read `crate::ui_profile::ui_profile()`. Under `Upstream`, produce the upstream rows (no icons, no forced bold).
- Also gate the `success_marker`/status glyph call sites already covered transitively by Task 2 (verify no residual inline literals here).
- Test: add dual-profile snapshot for the multi-row status surface; also honor the existing `status_line_use_colors` interaction (do not regress it).

- [ ] Steps: failing dual-profile status-surface snapshot → gate icons/bold on profile → upstream arm == upstream rows → PASS → fix/fmt/commit.

---

### Task 7: Add the `tui.ui_profile` config key (optional; env already works)

Graduates the enum to the config crate so the profile can be set via config/`-c` in addition to the env var. Do this after Tasks 2–6 so gating exists to exercise it.

**Files:**
- Move: `UiProfile` enum to `codex-rs/config/src/types.rs` (next to `Tui`, add `pub ui_profile: UiProfile` field, `#[serde(default)]`, `rename_all = "kebab-case"` so `codex-dev` deserializes); re-export from `codex-rs/tui/src/ui_profile.rs`.
- Modify: `codex-rs/config/src/types.rs:746` (`struct Tui`), `codex-rs/config/src/config_toml.rs:156/365` (already embeds `tui`), `codex-rs/core/src/config/mod.rs` (add `tui_ui_profile` flattened field + mapping at `:4395-4403`), `codex-rs/tui/src/local_settings.rs:17-18/54-69` (carry `ui_profile` into `LocalSettings.tui`).
- Modify: `codex-rs/tui/src/tui.rs` — resolve the profile from config first, fall back to env, then freeze the global (config wins).
- Run: `just write-config-schema` (updates `codex-rs/core/config.schema.json`) — required by AGENTS.md when `ConfigToml` changes. Run `just bazel-lock-update` only if deps changed (they should not).
- Test: config round-trip test that `tui.ui_profile = "codex-dev"` and `-c tui.ui_profile="codex-dev"` both resolve to `CodexDev`.

- [ ] Steps: failing config round-trip test → add field across the three layers → schema regen → PASS → fix/fmt/commit.

---

### Task 8: Crate-wide byte-identity gate + snapshot reconciliation

**Files:**
- Regenerate all existing `.snap` files under `codex-rs/tui/src/**/snapshots/` as the `CodexDev` render (they already encode the codex-dev look; re-accept after Tasks 2–6 with `cargo insta accept -p codex-tui`), and add the new `_upstream` snapshots from each task.
- Add a smoke test that renders a representative transcript+composer+status frame under `UiProfile::Upstream` and asserts it matches a snapshot captured from a clean upstream build (the load-bearing "default == upstream" proof).
- Verify the isolation gate elsewhere: `codex` resolves to upstream, only `codex-dev agents` to the custom build (belongs to the install/CI suite, not this crate).

- [ ] Steps: run `just test -p codex-tui`; review `cargo insta pending-snapshots -p codex-tui`; read each `.snap.new`; accept intentional; commit snapshots with the code that changed them.

---

## Self-review checklist (run after Tasks 2–8 land)

1. **Spec coverage:** every §4 mapping row marked L2 (groups A–I in the code map) is gated by some task. Gaps: none expected — A/C/D/I → Task 2/3; F/G → Task 4; H → Task 5; E → Task 6.
2. **Placeholder scan:** no "the upstream style" left unresolved — each such site cites the exact `git show upstream/main:<path>` baseline.
3. **Type consistency:** the getter is `ui_profile() -> UiProfile` everywhere; variants are `Upstream`/`CodexDev`; no bool parameters introduced.
4. **Invariant:** with `UiProfile::Upstream`, no task's `_upstream` snapshot differs from upstream bytes.

## Exit criterion

Default (`Upstream`) render is byte-identical to upstream (all `_upstream` snapshots clean); `codex-dev agents` (launcher exports `CODEX_UI_PROFILE=codex-dev`) renders the full customization set; every customization is covered by a `_codex_dev` snapshot; upstream `.snap` files are no longer edited during a sync because the default path already matches them.
