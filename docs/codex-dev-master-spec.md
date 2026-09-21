# `codex-dev` — Master Product & Engineering Spec

**Status:** Draft 1 · **Date:** 2026-09-21 · **Supersedes:** framing of `docs/codex-dev-implementation-plan.md` (that plan becomes the Phase-1 execution detail under §12) · **Owner:** integration owner of `integration/codex-dev`

## 0. Purpose & North Star

`codex-dev` is a customized distribution of OpenAI's Codex CLI whose only user-facing surface is `codex-dev agents`. This spec defines how to ship a first-class, best-in-class build with no bolt-on patch rot.

**North star:** `codex-dev` is a plugin/config-first product over *unmodified* official codex, plus a minimal, first-class, individually-droppable compiled renderer layer gated behind a runtime toggle, fed by an upstream-first contribution pipeline whose explicit goal is to shrink the compiled layer toward zero.

**Non-goals:** a hard fork that drifts from upstream; un-gated/un-tested/un-tracked source edits; a from-scratch frontend (see §7, rejected); changing official `codex`.

## 1. Current State (snapshot 2026-09-21)

> Note: the installed baseline is a **moving target** while concurrent sessions land green-marker refinements on integration; validation must freeze the branch/install pair and hash-pin per §8. Recompute this table at each sync.

| Item | Value |
|---|---|
| Integration tip | `b468417472` (branch `integration/codex-dev`; adds `scripts/codex-dev-verify-installed.sh` on top of installed code commit `a9bdcba039`) |
| Installed baseline | `a9bdcba039` (receipt hash `a0ed3aa6b91d812b2394878b7d0e7a48d0bbe8b7d5371c1faeec02dbd1f96e2a`, installed 2026-09-21T06:54:50Z) |
| Upstream merge-base | `e269f2164c` |
| Downstream delta | 52 commits ahead |
| TUI footprint | ~355 `tui/src` paths ≈ 118 source + 237 snapshots (recount at each sync) |
| Non-TUI footprint | small: `core/src`, `config/src`, docs, scripts |
| History shape | merge-based (contains merge commits) |
| Toolchain | pinned Rust 1.95.0 (`codex-rs/rust-toolchain.toml`); Rust 1.98 hit an async query-depth limit |
| Official codex | separate, unmodified, at 0.155.1 (`/opt/homebrew/bin/codex`) |
| Runtime UI gating | none — all customization is un-gated compiled divergence (no renderer refactor yet) |
| Launcher | `~/.local/bin/codex-dev` wrapper is not version-controlled |
| In-flight edits | landed: preserved `agents_overview_view.rs` + `history_cell/base.rs` committed at `1d10ccf90f`; multiple green success-marker refinements since |
| Release ledger | first installed-baseline row recorded in `docs/codex-dev-upstream-update-plan.md`; Ghostty smoke still pending against a frozen, hash-pinned pair |
| Validation infra | `scripts/codex-dev-verify-installed.sh` landed at `b468417472` — verifies branch, receipt commit validity, receipt SHA-256, installed binary hash, and installed-commit-is-ancestor-of-HEAD; passes against `a9bdcba039` / `a0ed3aa6…` |

**Trend:** the delta is growing sync-over-sync (44 → 45 → 52). Reversing that trend is the core objective of this spec.

## 2. Contract (hard constraints — every decision preserves these)
1. Official `codex` stays upstream-identical (entrypoint, subcommands, and rendered output).
2. Only `codex-dev agents` is user-facing.
3. `integration/codex-dev` is the sole release branch; never force-push it.
4. Build + install only from `.worktrees/integration` via `scripts/install-codex-dev.sh`; one integration owner; feature sessions never install.
5. Every change is a categorized commit (upstream-candidate / config-plugin-candidate / downstream-UX / compat-fix) — never one flattened merge.
6. Linux/macOS/Windows unless explicitly OS-specific; never touch `CODEX_SANDBOX*`.

## 3. Architecture

Three components with a one-directional "graduation" flow (L2 → seam → L1/upstream):

- **Layer 1 — Product layer (zero source patch):** a `codex-dev` plugin/config package that rides *unmodified* official `codex`. Holds all behavior + the thin slice of appearance upstream actually exposes.
- **Layer 2 — Renderer layer (minimal compiled patch):** the irreducible visual identity, gated behind a runtime toggle so the default render path is byte-identical to upstream.
- **Seam pipeline:** upstream PRs that add the extension points which let L2 items graduate to L1. The metric of success is L2 shrinking over time.

## 4. Layer Boundary — Decision Rule + Mapping

**Decision rule (write into the manifest):** a customization may live in Layer 2 only if it cannot be expressed as (a) official config, (b) a plugin, or (c) a value fed into an upstream-accepted seam. Everything else is Layer 1. Re-test every L2 item each sync.

**Grounded mapping (verified against upstream config-reference + `codex-rs/tui` source + issues):**

| Customization | Upstream surface today? | Layer |
|---|---|---|
| Skills, custom commands, hooks, MCP/apps, agent roles | Plugin system (behavior only) | L1 |
| Syntax-highlight palette | `tui.theme` (`.tmTheme`; also tints status line + inline code) | L1 (bundle a custom `.tmTheme`) |
| Status-line item selection/order | `tui.status_line` (fixed built-in ids only) | L1 |
| Keymap, terminal title, notifications, animations, alt-screen | `[tui]` config keys | L1 |
| Agents-overview orchestration/project-selection logic | none (compiled), but partly expressible via command/app/hook | L1 where behavioral, else L2 |
| Multi-row `status_lines` + custom items (`five-hour-limit`, `estimated-thread-cost`, `run-state`, `pull-request-number`…) | none — not upstream's fixed set | L2 (upstream candidate) |
| Maroon/bold headings, distinct user/assistant styling, full-width user bands, transcript bottom borders, green success markers | none — hard-coded semantic/chrome colors | L2 (blocked on #21130/#17879) |
| Bold status rows + Nerd-Font icons, blue composer borders, sticky composers | none — compiled chrome | L2 |

**Verdict:** `codex-dev`'s entire visual identity is genuinely L2 today. Behavior + syntax theme + `[tui]` knobs move to L1. The visual identity stays a compiled patch until upstream lands semantic-color config.

## 5. Layer 1 Spec (what leaves the fork)
- Package: `codex-dev-plugin/` with `.codex-plugin/plugin.json` + `skills/`, `commands/`, `hooks.json`, `.mcp.json`/`.app.json`, `agents/`, `themes/<codex-dev>.tmTheme`.
- Riding unmodified `codex`: `codex-dev agents` becomes a launcher that runs official `codex` with the plugin/config active (a bundled project/user `config.toml` selecting the `.tmTheme`, `status_line` items, keymap, terminal title).
- **Acceptance:** at least the syntax theme + status-item selection + one behavioral feature (project-selector logic as a command/hook, or MCP) run over *official* `codex` with zero source patch. Prove `codex` unchanged; `codex-dev agents` resolves to the custom path.

## 6. Layer 2 Spec (irreducible compiled patch)
- **Theming architecture:** introduce a `Theme`/`UiProfile { Upstream, CodexDev }` enum (no bool params — AGENTS.md) read once at startup from a config key (`ConfigToml`, then `just write-config-schema`) and/or env (`CODEX_DEV_UI`); the launcher sets it. Thread it through `style.rs`, `history_cell/`, `chatwidget/rendering.rs`, `bottom_pane/` (composer + status line), `app/agents_overview_render.rs`.
- **Gating invariant:** toggle OFF ⇒ render output byte-identical to upstream (upstream `.snap` pass unmodified). Toggle ON ⇒ codex-dev identity, covered by new, separate `.snap` files.
- **Snapshot strategy:** never edit upstream snapshots in place; add codex-dev snapshots as new `#[path=...]` modules. This collapses the 237-file snapshot conflict tax at each sync.
- **Already landed (to be moved behind the toggle):** the preserved `agents_overview_view.rs` + `history_cell/base.rs` (committed `1d10ccf90f`) and the green success-marker refinements are un-gated L2 today; migrate them behind the toggle as it is introduced.

## 7. Upstream Contribution Pipeline (the flagship move)
- **Highest-leverage seam:** a semantic-color / TUI-chrome theming config surface resolving upstream #21130 (semantic colors) + #17879 (distinct user/assistant styling). If accepted, the entire L2 visual identity collapses to a bundled theme + config (graduates to L1). Upstream is already drifting this way (merged PRs #46504 theme-aware accents, #19631 statusline from theme, #46069 syntax colors for inline code/paths), so appetite exists.
- **Second seam:** a multi-row / custom-item status-line capability (note #20244→#17827 rejected command-backed status lines — propose structured multi-row built-in items, not arbitrary ANSI/commands).
- **Standalone feature PR:** the agents project-directory selector (spec at `docs/agents-project-directory-selector-spec.md`) — if merged, its branch drops from the stack.
- **Graduation discipline (nixpkgs/Asahi pattern):** each carried patch names its intent + tracks its upstream PR/issue; the instant it merges upstream, delete the downstream patch and rebase. Goal = delete patches, don't accumulate.
- **Rejected alternative — app-server frontend:** the app-server v2 JSON-RPC API (80+ methods, Thread/Turn/Item, streamed deltas) could power a from-scratch frontend, but it exposes only data/deltas/approvals — you'd rebuild all rendering and forfeit upstream TUI improvements. Documented and rejected in favor of the gated-patch + seam path.

## 8. Sync & Release Engineering
- **Model: rebase patch-stack, not merge** (Igalia/Chromium, ungoogled-chromium, Asahi precedent). Because `integration/codex-dev` is shared and must not be force-pushed, use the no-force-push hybrid: in a throwaway worktree, `git merge -s ours upstream/<tag>` so the tree matches upstream, fast-forward, then cherry-pick only still-relevant categorized patches; `git log upstream..HEAD --no-merges` is your true delta.
- **Tooling:** `rerere` (already on) replays recurring conflict resolutions; `range-diff` proves a sync changed only what conflict resolution required and shows which patches dropped because they're now upstream. Consider StGit/quilt-style `series` manifest for named, tracked patches.
- **CI gate — "series applies cleanly to pinned upstream"** (ungoogled-chromium validator pattern): fail the build if the stack no longer applies. Failure = a signal an upstream assumption changed, not a nuisance.
- **Cadence:** track upstream on a documented schedule (e.g., each upstream stable, promptly after security releases). Smaller, frequent bumps beat rare large ones.
- **Provenance ledger (self-maintaining):** `install-codex-dev.sh` appends a ledger row derived from its receipt (it already writes commit + sha256 + date); refresh the recorded baseline SHA after each sync. Columns: Date · Custom tag · Integration commit · Upstream commit · Official stable observed · Tests · Ghostty smoke test.
- **Validation-window serialization (learned in practice):** the installed baseline advanced `1d10ccf90f → 805bcb2a0e → b2a2ed5aee → a9bdcba039` while validations were running, invalidating in-flight smokes. For any validation window the integration owner takes an explicit "validation frozen" hold; other sessions stage on feature branches and do **not** merge to integration or install until the smoke verdict + ledger row are recorded. Pin validation to the **receipt hash**, not to integration HEAD (docs commits legitimately advance HEAD past the installed code commit).

## 9. Build, Toolchain & Provenance
- **Toolchain pin + the 1.98 async query-depth risk:** keep the 1.95.0 pin. The 1.98 failure is a fragility signal — the codebase has deeply-nested async that trips the compiler's query/recursion depth on newer toolchains, and it corroborates the stack-overflow smell (§10.1). **Risk:** when upstream bumps its toolchain, the fork inherits this. **Actions:** (a) reduce async nesting depth (apply AGENTS.md's native-RPITIT + explicit-`Send` guidance; avoid `#[async_trait]` layering; flatten deeply-nested futures); (b) add a CI job building against the next stable toolchain as allowed-failure for early warning; (c) treat `#![recursion_limit]`/`#![type_length_limit]` bumps as band-aids, not fixes.
- **Release artifact:** build `--release --locked`; profile `opt-level=3, lto="fat"*, codegen-units=1, panic="abort"*, strip=true` and archive split-debuginfo separately for symbolication (this is exactly what `[profile.release]`'s own comment intends but the install script currently skips — a prior installed 299 MB unstripped binary is the bug). `*` verify `lto=fat` gains by benchmark and that no path relies on unwinding before `panic=abort` (the terminal-restore panic hook still runs before abort, so restore is safe — only `catch_unwind`-based recovery is lost).
- **Version stamping (fixes "`--version` indistinguishable from official"):** use `vergen` (v9: `vergen-git2`/`gitcl`/`gix`) to stamp `codex-dev --version` with custom tag + upstream base SHA + integration commit + dirty flag + build date.
- **Reproducible + signed:** `--locked`, pinned toolchain, `SOURCE_DATE_EPOCH`, `trim-paths`/`--remap-path-prefix`; GitHub Artifact Attestations (keyless SLSA provenance via `actions/attest-build-provenance`) + cosign signing of archives + an SBOM; document the `gh attestation verify` command.

## 10. Reliability & Hardening
1. **Root-cause the stack overflow; delete `RUST_MIN_STACK`:** it's almost certainly unbounded recursion in content rendering (markdown/AST/syntax-highlight) or a recursive `Drop`, and it doesn't even cover tokio workers (2 MB default). Capture the backtrace with the override removed, rewrite the walk iteratively, and bound rendered content nesting depth (model output can be adversarially deep — a DoS vector). Ties to §9's async-depth issue.
2. **Terminal never corrupts:** chain the panic hook through `ratatui::restore()` then delegate to color-eyre; add `SIGINT/SIGTERM/SIGHUP` handling via a `signal-hook` thread → `restore()` → re-raise (a panic hook alone misses SIGTERM/SIGHUP).
3. **Unicode/width + terminals:** stay on ratatui ≥0.29/0.30 (buffer-panic fixes), guard 0-size areas, handle resize; gate the Nerd-Font icons behind capability/opt-out with an ASCII fallback (PUA glyphs render as 1/2 cells or tofu); detect truecolor via terminfo/OSC when `COLORTERM` is stripped over SSH (upstream #23677), `NO_COLOR` first.
4. **Performance/scrollback:** do markdown parse/highlight/wrap off the draw thread; store pre-wrapped lines and render only the visible slice, cache by width, cap scrollback with a ring buffer (relevant given 326 MB-scale rollouts).
5. **Session-store durability + resume-compat:** add retention/rotation + size caps; add a resume-compat test that resumes a pre-sync rollout after every sync (AGENTS.md flags rollout resume as a breaking-change surface).
6. **Config validation:** reconcile the split status-line schema (config `status_line` vs launcher `-c tui.status_lines=[[…]]`) into validated config; fail loudly on malformed overrides.
7. **Preserve safety semantics:** verify `feat/user-approval-style` changed only presentation, not approval/sandbox semantics; add a test asserting approval gating equals upstream.
8. **Install/rollback:** verify the installed binary's sha == the tested sha (hash-pin, see §11); keep N rollback generations + `codex-dev --rollback`; sweep orphaned `*.stage.*` files; keep the prior binary until the new one passes the installed smoke.

## 11. Testing Strategy
- **Breadth:** `TestBackend` + `insta` frame snapshots at fixed dims; redactions/filters for non-determinism (note: TestBackend doesn't assert color).
- **Depth (E2E):** a handful of PTY flows (termlens, or expectrl/portable-pty + vt100) driving `codex-dev agents`: open, scroll, select, both composers, project selection, return-to-agents, resize; assert the verified behaviors. Mouse selection/copy is explicitly out-of-scope for automation → manual Ghostty checklist.
- **Property/fuzz:** `proptest` over word-wrap/width invariants with adversarial unicode (CJK width-2, ZWJ, VS16, combining marks, control chars).
- **Hash-pinned installed validation (foundation slice — LANDED `b468417472`):** the executable `scripts/codex-dev-verify-installed.sh` verifies branch, receipt commit validity, receipt SHA-256, the installed binary's hash (`shasum -a256` == receipt), and that the installed commit is an ancestor of integration HEAD; it passes against the current install (`a9bdcba039` / `a0ed3aa6…`). It runs as the first step of every smoke and exits non-zero on mismatch. This was the smallest safe slice and is the shared foundation for install-time hash-verify, rollback keying, ledger rows, and the PTY suite.
- **Gates:** command-isolation test (`codex` == upstream, only `codex-dev agents` == custom); the series-applies gate (§8); resume-compat (§10.5); build against next-stable toolchain (allowed-failure, §9). All required on PRs into `integration/codex-dev`.

## 12. Migration Roadmap (aligned to current state)
0. **Done:** Phase-1 inventory persisted (`docs/codex-dev-implementation-plan.md`); rebuild on pinned 1.95; first installed-baseline ledger row recorded; the §11 hash-verify foundation slice (`scripts/codex-dev-verify-installed.sh`) landed at `b468417472`.
1. **Done:** the preserved `agents_overview_view.rs` + `history_cell/base.rs` committed at `1d10ccf90f` (plus subsequent green success-marker refinements). A first install has been taken (currently `a9bdcba039`); **Ghostty smoke is pending against a frozen, hash-pinned pair** (mouse open/click, selection/copy, Jump-to-bottom click, return-to-agents, resize).
2. **Introduce the `UiProfile` toggle** (§6) — default OFF = upstream-identical; migrate the already-landed L2 (agents/history-cell/green markers) behind it, staged per render area (<800 lines each). Do this before the 0.155.1 sync so the snapshot conflict tax drops.
3. **Version the launcher + release-build hygiene** (§9) before the next fresh install: stripped `--release`, `vergen` stamping, hash-verified install, self-maintaining ledger.
4. **Extract Layer 1** (§5): plugin/config package + bundled `.tmTheme`; prove behaviors ride unmodified `codex`.
5. **Automate CI gates** (§11) including the series-applies and isolation tests, built on the §11 hash-verify foundation slice.
6. **Adopt the rebase hybrid sync** (§8) and retire the merge model.
7. **Open the flagship seam PRs** (§7: #21130/#17879 semantic colors; multi-row status) and drop each downstream patch as it graduates.

**Exit criterion for "best-in-class":** default path == upstream (snapshots clean); L2 fully gated + covered; `--version` self-identifying; stripped/signed release with provenance; `RUST_MIN_STACK` deleted; ≥1 seam PR open; delta measurably shrinking sync-over-sync.

## 13. Risks & Open Questions
- **Toolchain lock-in (1.95):** upstream toolchain bump could force the async-depth fix under time pressure — mitigate now via §9 actions.
- **Upstream may reject the semantic-color seam** — keep the toggle working regardless; L2 must never depend on upstream acceptance.
- **`panic=abort` unwinding** — verify no `catch_unwind` dependency before adopting.
- **cargo-dist maintenance** — adopt for CI generation but keep the generated `release.yml` under your own control.
- **Concurrent-session serialization** — the installed baseline is a moving target; enforce the §8 validation-window freeze so smokes are not invalidated mid-run.
- **Open:** exact upstream item-id set for a proposed multi-row status line; whether agents-overview logic can be a hook/app vs. compiled.

## 14. Appendix
- **Key file map:** renderer L2 — `codex-rs/tui/src/{style.rs, color.rs, history_cell/, chatwidget/rendering.rs, bottom_pane/(chat_composer.rs,footer.rs,mod.rs,status_line_style.rs), app/agents_overview_render.rs}`. Install/provenance — `scripts/{install-codex-dev.sh, test_install_codex_dev.py}` + new `scripts/codex-dev` launcher + `scripts/codex-dev-verify-installed.sh` (landed `b468417472`). Docs — `docs/codex-dev-*`.
- **Primary citations:** upstream issues #21130, #17879, #20244→#17827, #23677; config-reference (`learn.chatgpt.com/docs/config-file/config-reference`), app-server README; Igalia downstream-strategy post; ratatui panic/testing recipes; Rust perf-book release profiles; GitHub Artifact Attestations / Sigstore.
