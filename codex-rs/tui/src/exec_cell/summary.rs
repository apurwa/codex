//! Compact successful shell activity; the transcript overlay retains commands and output.

use super::model::ExecCell;
use crate::exec_command::strip_bash_lc_and_escape;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use codex_utils_elapsed::format_duration;
use ratatui::prelude::*;

impl ExecCell {
    pub(super) fn completed_summary(&self, width: u16) -> Option<Vec<Line<'static>>> {
        if self.calls.is_empty()
            || self.calls.iter().any(|call| {
                call.is_user_shell_command()
                    || call.duration.is_none()
                    || call
                        .output
                        .as_ref()
                        .is_none_or(|output| output.exit_code != 0)
            })
        {
            return None;
        }

        let title = if self.is_exploring_cell() && self.calls.len() > 1 {
            format!("Explored · {} commands", self.calls.len())
        } else if self.is_exploring_cell() {
            "Explored".to_string()
        } else if self.calls.len() > 1 {
            format!("Ran · {} commands", self.calls.len())
        } else {
            let script = strip_bash_lc_and_escape(&self.calls[0].command);
            let first_line = script
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("Command");
            // Decode terminal control sequences before building a single-line summary.
            codex_ansi_escape::ansi_escape_line(first_line).to_string()
        };
        let duration = self.calls.iter().filter_map(|call| call.duration).sum();
        let output_lines: usize = self
            .calls
            .iter()
            .filter_map(|call| call.output.as_ref())
            .map(|output| output.line_counts().0)
            .sum();
        let details = if width >= 65 {
            format!(
                " · {} · {output_lines} lines · Ctrl+T details",
                format_duration(duration)
            )
        } else {
            " · Ctrl+T".to_string()
        };
        let title_width = usize::from(width).saturating_sub(2 + details.chars().count());
        let mut line = Line::from(vec![crate::style::success_marker("✓ ")]);
        line.extend(
            truncate_line_with_ellipsis_if_overflow(Line::from(title).bold(), title_width).spans,
        );
        line.push_span(details);
        Some(vec![truncate_line_with_ellipsis_if_overflow(
            line,
            usize::from(width),
        )])
    }
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
