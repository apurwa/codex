//! Runtime UI profile toggle for the `codex-dev` build.
//!
//! `codex-dev` ships downstream visual customizations (maroon headings, full-width
//! user bands, green success markers, and custom composer/status chrome) that
//! upstream Codex does not expose as configuration. To keep the default render path
//! byte-identical to upstream — and to keep upstream syncs cheap — those
//! customizations are gated behind this profile. The `codex-dev` launcher exports
//! [`CODEX_UI_PROFILE_ENV_VAR`]`=codex-dev`; every other invocation defaults to
//! [`UiProfile::Upstream`], so a binary built from this fork renders exactly like
//! upstream unless the launcher opts in.
//!
//! The profile is process-global state resolved once at startup, mirroring
//! [`crate::terminal_palette`]'s startup-probe pattern. Storing it globally lets the
//! free styling helpers in [`crate::style`] consult it without threading a parameter
//! through every render call site. Rendering code must read it only through
//! [`ui_profile`] so the `#[cfg(test)]` thread-local override applies.
//!
//! This module is deliberately render-neutral: it establishes and exposes the
//! switch. Individual customizations are moved behind it in later, separately
//! reviewed changes, each proving that [`UiProfile::Upstream`] reproduces upstream
//! output byte-for-byte.

use std::sync::OnceLock;

/// Selects which visual identity the TUI renders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UiProfile {
    /// Render exactly as upstream Codex does, with no downstream customization.
    #[default]
    Upstream,
    /// Render the `codex-dev` visual identity.
    CodexDev,
}

/// Environment variable the `codex-dev` launcher exports to select the profile.
///
/// The value `codex-dev` selects [`UiProfile::CodexDev`]; any other value, or an
/// unset variable, selects [`UiProfile::Upstream`].
pub(crate) const CODEX_UI_PROFILE_ENV_VAR: &str = "CODEX_UI_PROFILE";

impl UiProfile {
    fn from_env_value(value: &str) -> Self {
        match value.trim() {
            "codex-dev" | "codex_dev" | "codexdev" => Self::CodexDev,
            _ => Self::Upstream,
        }
    }
}

static UI_PROFILE: OnceLock<UiProfile> = OnceLock::new();

#[cfg(test)]
thread_local! {
    static TEST_UI_PROFILE: std::cell::Cell<Option<UiProfile>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
tokio::task_local! {
    static ASYNC_TEST_UI_PROFILE: UiProfile;
}

/// Resolve the profile from the environment and freeze it for the process.
///
/// Called once during TUI startup, before any rendering. The value is stored in a
/// [`OnceLock`], so later calls are ignored and the profile cannot change
/// mid-session.
pub(crate) fn set_ui_profile_from_env() {
    let profile = std::env::var(CODEX_UI_PROFILE_ENV_VAR)
        .ok()
        .map_or(UiProfile::Upstream, |value| {
            UiProfile::from_env_value(&value)
        });
    let _ = UI_PROFILE.set(profile);
}

/// The active UI profile.
///
/// Defaults to [`UiProfile::Upstream`] until [`set_ui_profile_from_env`] runs, so
/// rendering that happens before startup (or in code paths that never initialize the
/// TUI) stays upstream-identical.
pub(crate) fn ui_profile() -> UiProfile {
    #[cfg(test)]
    if let Ok(profile) = ASYNC_TEST_UI_PROFILE.try_with(|profile| *profile) {
        return profile;
    }
    #[cfg(test)]
    if let Some(profile) = TEST_UI_PROFILE.with(std::cell::Cell::get) {
        return profile;
    }
    UI_PROFILE.get().copied().unwrap_or_default()
}

/// Scope a profile across an async test future. Unlike the synchronous helper,
/// this uses Tokio task-local state so the override remains active across `.await`
/// points and follows the future if Tokio moves it between worker threads.
#[cfg(test)]
pub(crate) async fn with_test_ui_profile_async<F, T>(profile: UiProfile, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    ASYNC_TEST_UI_PROFILE.scope(profile, future).await
}

/// Scope a [`UiProfile`] to the current test thread while rendering a widget.
///
/// This mirrors [`crate::terminal_palette::with_test_default_colors`] so snapshot
/// tests can assert both the `Upstream` (byte-identical to upstream) and `CodexDev`
/// renders from a single test without mutating process-global state.
#[cfg(test)]
pub(crate) fn with_test_ui_profile<T>(profile: UiProfile, render: impl FnOnce() -> T) -> T {
    TEST_UI_PROFILE.with(|slot| {
        let previous = slot.replace(Some(profile));
        let result = render();
        slot.set(previous);
        result
    })
}

#[cfg(test)]
#[path = "ui_profile_tests.rs"]
mod tests;
