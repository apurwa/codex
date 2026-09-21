use crate::color::blend;
use crate::color::is_light;
use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::best_color;
use crate::terminal_palette::default_bg;
use crate::terminal_palette::default_fg;
use crate::terminal_palette::effective_stdout_color_level;
use crate::terminal_palette::rgb_color;
use crate::terminal_palette::stdout_color_level;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

const LIGHT_BG_ACCENT_RGB: (u8, u8, u8) = (0, 95, 135);
pub(crate) const COMPOSER_BLUE_RGB: (u8, u8, u8) = (0, 95, 135);

/// Speaker label using the shared Codex response accent.
pub(crate) fn codex_label_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => codex_response_border_style().bold(),
        crate::ui_profile::UiProfile::Upstream => Style::default(),
    }
}

/// Cool assistant-response accent shared by Codex cards and the working shimmer.
pub(crate) fn composer_blue_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => Style::default().fg(Color::Rgb(
            COMPOSER_BLUE_RGB.0,
            COMPOSER_BLUE_RGB.1,
            COMPOSER_BLUE_RGB.2,
        )),
        crate::ui_profile::UiProfile::Upstream => Style::default(),
    }
}

/// Border style for Codex response cards.
pub(crate) fn codex_response_border_style() -> Style {
    composer_blue_style().bold()
}

/// Pale cool background that distinguishes Codex responses from user messages.
pub(crate) fn codex_response_background_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => match default_bg() {
            Some(bg) => Style::default().bg(best_color(codex_response_bg_rgb(bg))),
            None => Style::default(),
        },
        crate::ui_profile::UiProfile::Upstream => Style::default(),
    }
}

pub(crate) fn codex_response_rule(width: u16) -> Line<'static> {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => Line::from(Span::styled(
            "─".repeat(usize::from(width.saturating_sub(1))),
            codex_response_border_style(),
        )),
        crate::ui_profile::UiProfile::Upstream => Line::default(),
    }
}

/// High-contrast success accent for glyphs shown on light transcript backgrounds.
pub(crate) fn tool_success_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => {
            // Use a vivid green on the light transcript background; the previous
            // forest green read as olive/muted in the pale terminal theme.
            Style::default().fg(Color::Rgb(0x00, 0xc8, 0x53)).bold()
        }
        crate::ui_profile::UiProfile::Upstream => Style::default().fg(Color::Green).bold(),
    }
}

/// Render a success marker consistently across transcript, approval, and status surfaces.
pub(crate) fn success_marker(text: &'static str) -> Span<'static> {
    Span::styled(text, tool_success_style())
}

pub(crate) fn success_marker_with_upstream(
    text: &'static str,
    upstream: Span<'static>,
) -> Span<'static> {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => success_marker(text),
        crate::ui_profile::UiProfile::Upstream => upstream,
    }
}

/// Shared selection/action colors, independent of remapped ANSI terminal palettes.
pub(crate) fn selected_control_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(0, 95, 135))
            .remove_modifier(ratatui::style::Modifier::DIM | ratatui::style::Modifier::REVERSED),
        crate::ui_profile::UiProfile::Upstream => Style::default(),
    }
}

#[derive(Clone, Copy)]
pub(crate) enum StatusTone {
    Success,
    Attention,
    Failure,
}

/// Semantic status colors that preserve the terminal's configured palette.
pub(crate) fn status_style(tone: StatusTone) -> Style {
    status_style_for(tone, default_bg(), effective_stdout_color_level())
}

fn status_style_for(
    tone: StatusTone,
    terminal_bg: Option<(u8, u8, u8)>,
    color_level: StdoutColorLevel,
) -> Style {
    let light = terminal_bg.is_some_and(is_light);
    let color = match (tone, color_level) {
        (_, StdoutColorLevel::Unknown) => Color::Reset,
        (StatusTone::Success, _) => Color::Green,
        (StatusTone::Failure, _) => Color::Red,
        // Yellow can disappear on light themes; use it only with a known dark background.
        (StatusTone::Attention, _) if light || terminal_bg.is_none() => Color::Reset,
        (StatusTone::Attention, _) => Color::Yellow,
    };
    Style::default().fg(color).bold()
}
// Decorative table rules should remain visible without competing with cell content.
const TABLE_SEPARATOR_FG_ALPHA: f32 = 0.20;

pub fn user_message_style() -> Style {
    user_message_style_for(default_bg())
}

/// Returns the stronger background used specifically for user-authored transcript cells.
pub(crate) fn transcript_user_message_style() -> Style {
    match crate::ui_profile::ui_profile() {
        crate::ui_profile::UiProfile::CodexDev => match default_bg() {
            Some(bg) => Style::default().bg(best_color(transcript_user_message_bg_rgb(bg))),
            None => Style::default(),
        },
        crate::ui_profile::UiProfile::Upstream => Style::default(),
    }
}

pub fn proposed_plan_style() -> Style {
    proposed_plan_style_for(default_bg())
}

/// Returns a low-contrast rule style for separators within markdown tables.
pub(crate) fn table_separator_style() -> Style {
    table_separator_style_for(default_fg(), default_bg(), stdout_color_level())
}

/// Returns the shared accent style for active or selected TUI controls.
pub(crate) fn accent_style() -> Style {
    accent_style_for(default_bg())
}

pub(crate) fn footer_hint_key_style() -> Style {
    if default_bg().is_some_and(is_light) {
        Style::default().fg(Color::Black)
    } else {
        Style::default()
    }
}

pub(crate) fn footer_hint_label_style() -> Style {
    if default_bg().is_some_and(is_light) {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().dim()
    }
}

/// Returns the style for a user-authored message using the provided terminal background.
pub fn user_message_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(user_message_bg(bg)),
        None => Style::default(),
    }
}

pub fn proposed_plan_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(proposed_plan_bg(bg)),
        None => Style::default(),
    }
}

/// Returns the shared accent style for the provided terminal background.
pub(crate) fn accent_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_ACCENT_RGB)).bold()
    } else {
        Style::default().fg(Color::Cyan).bold()
    }
}

fn codex_response_bg_rgb(terminal_bg: (u8, u8, u8)) -> (u8, u8, u8) {
    let alpha = if is_light(terminal_bg) { 0.08 } else { 0.24 };
    blend(COMPOSER_BLUE_RGB, terminal_bg, alpha)
}

fn table_separator_style_for(
    terminal_fg: Option<(u8, u8, u8)>,
    terminal_bg: Option<(u8, u8, u8)>,
    color_level: StdoutColorLevel,
) -> Style {
    let (Some(fg), Some(bg)) = (terminal_fg, terminal_bg) else {
        return Style::default().dim();
    };
    let separator_rgb = blend(fg, bg, TABLE_SEPARATOR_FG_ALPHA);
    match color_level {
        StdoutColorLevel::TrueColor => Style::default().fg(rgb_color(separator_rgb)),
        StdoutColorLevel::Ansi256 => Style::default().fg(best_color(separator_rgb)),
        StdoutColorLevel::Ansi16 | StdoutColorLevel::Unknown => Style::default().dim(),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    best_color(user_message_bg_rgb(terminal_bg))
}

pub(crate) fn user_message_bg_rgb(terminal_bg: (u8, u8, u8)) -> (u8, u8, u8) {
    let (top, alpha) = if is_light(terminal_bg) {
        ((0, 0, 0), 0.04)
    } else {
        ((255, 255, 255), 0.12)
    };
    blend(top, terminal_bg, alpha)
}

fn transcript_user_message_bg_rgb(terminal_bg: (u8, u8, u8)) -> (u8, u8, u8) {
    let (top, alpha) = if is_light(terminal_bg) {
        ((0, 0, 0), 0.08)
    } else {
        ((255, 255, 255), 0.18)
    };
    blend(top, terminal_bg, alpha)
}

#[allow(clippy::disallowed_methods)]
pub fn proposed_plan_bg(terminal_bg: (u8, u8, u8)) -> Color {
    user_message_bg(terminal_bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Modifier;

    #[test]
    fn status_colors_preserve_light_terminal_themes() {
        for level in [StdoutColorLevel::TrueColor, StdoutColorLevel::Ansi256] {
            for bg in [(255, 255, 255), (130, 130, 130), (220, 210, 180)] {
                for (tone, color) in [
                    (StatusTone::Success, Color::Green),
                    (StatusTone::Attention, Color::Reset),
                    (StatusTone::Failure, Color::Red),
                ] {
                    assert_eq!(
                        status_style_for(tone, Some(bg), level),
                        Style::default().fg(color).bold(),
                    );
                }
            }
            for bg in [(0, 0, 0), (0, 218, 0)] {
                assert_eq!(
                    status_style_for(StatusTone::Attention, Some(bg), level),
                    Style::default().fg(Color::Yellow).bold(),
                );
            }
            assert_eq!(
                status_style_for(StatusTone::Attention, /*terminal_bg*/ None, level),
                Style::default().fg(Color::Reset).bold(),
            );
        }
    }

    #[test]
    fn status_colors_preserve_ansi16_and_no_color_fallbacks() {
        for (tone, light, dark) in [
            (StatusTone::Success, Color::Green, Color::Green),
            (StatusTone::Attention, Color::Reset, Color::Yellow),
            (StatusTone::Failure, Color::Red, Color::Red),
        ] {
            for (bg, expected) in [((255, 255, 255), light), ((0, 0, 0), dark)] {
                assert_eq!(
                    status_style_for(tone, Some(bg), StdoutColorLevel::Ansi16),
                    Style::default().fg(expected).bold()
                );
                assert_eq!(
                    status_style_for(tone, Some(bg), StdoutColorLevel::Unknown),
                    Style::default().fg(Color::Reset).bold()
                );
            }
        }
    }

    #[test]
    fn accent_style_uses_darker_cyan_on_light_backgrounds() {
        let style = accent_style_for(Some((255, 255, 255)));

        assert_eq!(style.fg, Some(best_color(LIGHT_BG_ACCENT_RGB)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn accent_style_uses_cyan_on_dark_or_unknown_backgrounds() {
        let expected = Style::default().fg(Color::Cyan).bold();

        assert_eq!(accent_style_for(Some((0, 0, 0))), expected);
        assert_eq!(accent_style_for(/*terminal_bg*/ None), expected);
    }

    #[test]
    fn table_separator_blends_toward_dark_background() {
        let style = table_separator_style_for(
            Some((255, 255, 255)),
            Some((0, 0, 0)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((51, 51, 51))));
    }

    #[test]
    fn table_separator_blends_toward_light_background() {
        let style = table_separator_style_for(
            Some((0, 0, 0)),
            Some((255, 255, 255)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((204, 204, 204))));
    }

    #[test]
    fn table_separator_dims_when_palette_aware_color_is_unavailable() {
        let expected = Style::default().dim();

        assert_eq!(
            table_separator_style_for(
                Some((255, 255, 255)),
                Some((0, 0, 0)),
                StdoutColorLevel::Ansi16,
            ),
            expected
        );
        assert_eq!(
            table_separator_style_for(
                /*terminal_fg*/ None,
                Some((0, 0, 0)),
                StdoutColorLevel::TrueColor,
            ),
            expected
        );
    }

    #[test]
    fn profile_gates_codex_identity_helpers() {
        let upstream =
            crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::Upstream, || {
                (
                    codex_label_style(),
                    tool_success_style(),
                    transcript_user_message_style(),
                    codex_response_rule(8),
                )
            });
        assert_eq!(upstream.0, Style::default());
        assert_eq!(upstream.1, Style::default().fg(Color::Green).bold());
        assert_eq!(upstream.2, Style::default());
        assert!(upstream.3.to_string().is_empty());

        let codex_dev =
            crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
                (
                    codex_label_style(),
                    tool_success_style(),
                    transcript_user_message_style(),
                    codex_response_rule(8),
                )
            });
        assert!(codex_dev.0.add_modifier.contains(Modifier::BOLD));
        assert_eq!(codex_dev.1.fg, Some(Color::Rgb(0, 200, 83)));
        assert_eq!(codex_dev.3.to_string(), "───────");
    }
}
