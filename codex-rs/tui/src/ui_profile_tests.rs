use super::UiProfile;
use super::ui_profile;
use super::with_test_ui_profile;
use pretty_assertions::assert_eq;

#[test]
fn codex_dev_values_select_codex_dev_profile() {
    for value in ["codex-dev", "codex_dev", "codexdev", "  codex-dev  "] {
        assert_eq!(UiProfile::from_env_value(value), UiProfile::CodexDev);
    }
}

#[test]
fn other_values_select_upstream_profile() {
    for value in ["", "upstream", "CODEX-DEV", "codex", "true", "1", "off"] {
        assert_eq!(UiProfile::from_env_value(value), UiProfile::Upstream);
    }
}

#[test]
fn default_profile_is_upstream() {
    // No thread-local override and no startup call in unit tests, so the global
    // stays unset and the profile falls back to upstream.
    assert_eq!(ui_profile(), UiProfile::Upstream);
}

#[test]
fn test_override_scopes_and_restores_the_profile() {
    assert_eq!(ui_profile(), UiProfile::Upstream);

    let observed = with_test_ui_profile(UiProfile::CodexDev, || {
        let outer = ui_profile();
        // A nested override restores the previous value when it returns.
        let inner = with_test_ui_profile(UiProfile::Upstream, ui_profile);
        (outer, inner)
    });
    assert_eq!(observed, (UiProfile::CodexDev, UiProfile::Upstream));

    // The override does not leak past its scope.
    assert_eq!(ui_profile(), UiProfile::Upstream);
}
