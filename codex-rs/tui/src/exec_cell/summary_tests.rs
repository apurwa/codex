use super::*;
use crate::exec_cell::CommandOutput;
use crate::exec_cell::new_active_exec_command;
use crate::history_cell::HistoryCell;
use codex_app_server_protocol::CommandExecutionSource;
use pretty_assertions::assert_eq;
use std::time::Duration;

#[test]
fn completed_commands_are_compact_without_losing_details() {
    let mut cell = new_active_exec_command(
        "test".into(),
        vec![
            "bash".into(),
            "-lc".into(),
            "printf 'first\\nsecond'".into(),
        ],
        Vec::new(),
        CommandExecutionSource::Agent,
        /*interaction_input*/ None,
        /*animations_enabled*/ false,
    );
    assert!(cell.completed_summary(80).is_none());
    cell.complete_call(
        "test",
        CommandOutput::new(0, "first\nsecond".into()),
        Duration::from_secs(2),
    );
    let raw = cell.raw_lines();
    for width in [4, 20, 80, 120] {
        let summary = cell.display_lines(width);
        assert_eq!(summary.len(), 3);
        assert_eq!(summary[1].to_string(), "CODEX · Tool Calls");
        assert!(summary[2].width() <= usize::from(width));
        assert_eq!(cell.raw_lines(), raw);
    }
    let summary = cell.display_lines(100);
    assert!(summary[2].to_string().contains("Ctrl+T details"));
    assert!(
        summary[2]
            .spans
            .iter()
            .all(|span| !span.style.add_modifier.contains(Modifier::DIM) || span.content == "┊ ")
    );
    let full = cell
        .transcript_lines(100)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(full.contains("first\nsecond"));
    insta::assert_snapshot!(
        "compact_command_and_details",
        format!("{}\n\n{full}", summary[2])
    );
}

#[test]
fn failed_and_user_shell_commands_keep_visible_output() {
    for (source, exit_code) in [
        (CommandExecutionSource::Agent, 1),
        (CommandExecutionSource::UserShell, 0),
    ] {
        let mut cell = new_active_exec_command(
            "test".into(),
            vec!["echo".into(), "important output".into()],
            Vec::new(),
            source,
            /*interaction_input*/ None,
            /*animations_enabled*/ false,
        );
        cell.complete_call(
            "test",
            CommandOutput::new(exit_code, "important output".into()),
            Duration::from_secs(1),
        );
        assert!(cell.completed_summary(80).is_none());
        assert!(
            cell.display_lines(80)
                .iter()
                .any(|line| line.to_string().contains("important output"))
        );
    }
}

#[test]
fn wide_character_commands_fit_narrow_summaries() {
    let mut cell = new_active_exec_command(
        "test".into(),
        vec!["echo".into(), "界".repeat(100)],
        Vec::new(),
        CommandExecutionSource::Agent,
        /*interaction_input*/ None,
        /*animations_enabled*/ false,
    );
    cell.complete_call(
        "test",
        CommandOutput::new(0, String::new()),
        Duration::from_secs(1),
    );
    for width in [8, 25, 65, 100] {
        assert!(cell.display_lines(width)[2].width() <= usize::from(width));
    }
}
