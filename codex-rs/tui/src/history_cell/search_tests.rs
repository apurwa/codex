use super::*;
use pretty_assertions::assert_eq;

#[test]
fn web_action_labels_and_missing_details() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let url = "https://example.com/docs";
        let actions = [
            WebSearchAction::OpenPage {
                url: Some(url.into()),
            },
            WebSearchAction::OpenPage { url: None },
            WebSearchAction::FindInPage {
                url: Some(url.into()),
                pattern: Some("needle".into()),
            },
            WebSearchAction::FindInPage {
                url: None,
                pattern: Some("needle".into()),
            },
            WebSearchAction::FindInPage {
                url: Some(url.into()),
                pattern: Some(String::new()),
            },
            WebSearchAction::FindInPage {
                url: None,
                pattern: None,
            },
        ];
        let rendered = actions
            .into_iter()
            .flat_map(|action| {
                new_web_search_call("call".into(), String::new(), action)
                    .display_lines(/*width*/ 80)
            })
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(rendered);
    });
}

#[test]
fn web_action_compact_display_preserves_transcript_and_raw_details() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let url = "https://example.com/docs/very-long-page?section=narrowing#details";
        let cell = new_web_search_call(
            "call".into(),
            String::new(),
            WebSearchAction::FindInPage {
                url: Some(url.into()),
                pattern: Some("日本語 🦀 needle".into()),
            },
        );
        let display = cell.display_lines(/*width*/ 32);
        assert_eq!(display.len(), 3);
        assert!(display.iter().all(|line| line.width() <= 32));
        insta::assert_snapshot!(
            display
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
        let full = format!("Searched for '日本語 🦀 needle' in {url}");
        assert_eq!(
            cell.raw_lines(),
            vec![
                Line::default(),
                Line::from("CODEX · Tool Calls"),
                Line::from(format!("┊ {full}")),
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
                format!("┊ • {full}"),
            ]
        );
    });
}

#[test]
fn pending_web_action_and_legacy_query() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cell = new_active_web_search_call(
            "call".into(),
            String::new(),
            /*animations_enabled*/ false,
        );
        insta::assert_snapshot!(
            cell.display_lines(/*width*/ 80)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            @r"

        CODEX · Tool Calls
        ┊ • Browsing the web"
        );
        let legacy = new_web_search_call("call".into(), "old query".into(), WebSearchAction::Other);
        assert_eq!(
            legacy.raw_lines(),
            vec![
                Line::default(),
                Line::from("CODEX · Tool Calls"),
                Line::from("┊ Searched the web for old query"),
            ]
        );
    });
}

#[test]
fn batched_search_retains_each_query() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cell = new_web_search_call(
            "call".into(),
            String::new(),
            WebSearchAction::Search {
                query: None,
                queries: Some(vec!["first query".into(), "second query".into()]),
            },
        );
        assert_eq!(
            cell.raw_lines(),
            vec![
                Line::default(),
                Line::from("CODEX · Tool Calls"),
                Line::from("┊ Searched the web for first query, second query"),
            ]
        );
        insta::assert_snapshot!(
            cell.display_lines(/*width*/ 80)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            @r"

        CODEX · Tool Calls
        ┊ • Searched the web for first query, second query"
        );
    });
}
