use super::*;
use pretty_assertions::assert_eq;

#[test]
fn viewed_image_retains_original_path_in_details() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        for path in [
            "/workspace/assets/example.png",
            r"C:\workspace\assets\example.png",
        ] {
            let cell = new_view_image_tool_call(LegacyAppPathString::from_string(path));
            assert_eq!(
                cell.display_lines(/*width*/ 80)
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                vec!["", "CODEX · Tool Calls", "┊ • Viewed image example.png"]
            );
            assert_eq!(
                cell.raw_lines(),
                vec![
                    Line::default(),
                    Line::from("CODEX · Tool Calls"),
                    Line::from(format!("┊ Viewed image {path}")),
                ]
            );
            assert_eq!(
                cell.transcript_lines(/*width*/ 200)
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                vec![
                    String::new(),
                    "CODEX · Tool Calls".to_string(),
                    format!("┊ • Viewed image {path}"),
                ]
            );
        }
    });
}

#[test]
fn viewed_image_narrow_summary() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cell = new_view_image_tool_call(LegacyAppPathString::from_string(
            "/workspace/very-long-screenshot-name.png",
        ));
        insta::assert_snapshot!(
            cell.display_lines(/*width*/ 32)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    });
}
