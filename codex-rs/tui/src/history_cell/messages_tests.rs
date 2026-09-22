use super::*;
use crate::history_cell::markdown_render_cache::MarkdownRenderCacheKey;
use assert_matches::assert_matches;
use pretty_assertions::assert_eq;

#[test]
fn transcript_structure_respects_ui_profile() {
    let user = UserHistoryCell {
        spoken: false,
        message: "hello".into(),
        text_elements: Vec::new(),
        local_image_paths: Vec::new(),
        remote_image_urls: Vec::new(),
    };

    let upstream =
        crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::Upstream, || {
            (
                user.display_lines(32),
                AgentMarkdownCell::new("answer".into(), Path::new("/tmp"))
                    .with_phase(Some(codex_protocol::models::MessagePhase::FinalAnswer))
                    .display_lines(32),
                user.background_style(),
                AgentMarkdownCell::new("answer".into(), Path::new("/tmp"))
                    .with_phase(Some(codex_protocol::models::MessagePhase::FinalAnswer))
                    .background_style(),
            )
        });
    assert!(!upstream.0.iter().any(|line| line.to_string() == "YOU"));
    assert!(
        !upstream
            .1
            .iter()
            .any(|line| line.to_string().contains("CODEX"))
    );
    assert_eq!(upstream.2, None);
    assert_eq!(upstream.3, None);

    let codex_dev =
        crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
            (
                user.display_lines(32),
                AgentMarkdownCell::new("answer".into(), Path::new("/tmp"))
                    .with_phase(Some(codex_protocol::models::MessagePhase::FinalAnswer))
                    .display_lines(32),
                user.background_style(),
                AgentMarkdownCell::new("answer".into(), Path::new("/tmp"))
                    .with_phase(Some(codex_protocol::models::MessagePhase::FinalAnswer))
                    .background_style(),
            )
        });
    assert!(codex_dev.0.iter().any(|line| line.to_string() == "YOU"));
    assert!(
        codex_dev
            .1
            .iter()
            .any(|line| line.to_string().contains("CODEX"))
    );
    assert!(codex_dev.2.is_some());
    assert!(codex_dev.3.is_some());
}

#[test]
fn user_message_has_matching_rules_without_changing_raw_text() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cell = UserHistoryCell {
            spoken: false,
            message: "First line\nSecond line".into(),
            text_elements: Vec::new(),
            local_image_paths: Vec::new(),
            remote_image_urls: Vec::new(),
        };
        let raw = cell.raw_lines();
        assert_eq!(
            raw.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            "First line\nSecond line"
        );
        for width in [12, 40, 100] {
            let lines = cell.display_lines(width);
            assert_eq!(lines.first(), lines.last());
            assert_eq!(lines[0].to_string(), "─".repeat(usize::from(width - 1)));
            assert_eq!(cell.transcript_lines(width), lines);
            assert_eq!(cell.raw_lines(), raw);
        }
    });
}

#[test]
fn commentary_and_final_answers_are_labeled_and_readable_with_unchanged_raw_source() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        use codex_protocol::models::MessagePhase;
        let source = "Checking the **rendering** path.";
        let commentary = AgentMarkdownCell::new(source.into(), Path::new("/tmp"))
            .with_phase(Some(MessagePhase::Commentary));
        let answer = AgentMarkdownCell::new(source.into(), Path::new("/tmp"))
            .with_phase(Some(MessagePhase::FinalAnswer));
        let commentary_lines = commentary.display_lines(40);
        assert_eq!(commentary_lines[1].to_string(), "CODEX · Update");
        assert_eq!(answer.display_lines(40)[1].to_string(), "CODEX · Answer");
        let streaming = AgentMessageCell::new(vec![source.into()], /*is_first_line*/ true);
        let headings = [
            commentary_lines.clone(),
            answer.display_lines(40),
            streaming.display_lines(40),
        ]
        .into_iter()
        .map(|lines| {
            let label = lines
                .iter()
                .flat_map(|line| &line.spans)
                .find(|span| span.content.starts_with("CODEX"))
                .expect("speaker heading");
            assert_eq!(label.style, crate::style::codex_label_style());
            assert!(label.style.add_modifier.contains(Modifier::BOLD));
            format!("{:?}", label.style)
        })
        .collect::<Vec<_>>();
        insta::assert_snapshot!(headings.join("\n"), @r###"
Style::new().fg(Color::Rgb(128, 0, 0)).bold()
Style::new().fg(Color::Rgb(128, 0, 0)).bold()
Style::new().fg(Color::Rgb(128, 0, 0)).bold()
"###);
        assert!(
            commentary_lines
                .iter()
                .flat_map(|line| &line.spans)
                .filter(
                    |span| span.content.contains("Checking") || span.content.contains("rendering")
                )
                .all(|span| !span.style.add_modifier.contains(Modifier::DIM))
        );
        assert_eq!(commentary.raw_lines(), answer.raw_lines());
        let rendered = commentary_lines
            .into_iter()
            .chain(answer.display_lines(40))
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!("commentary_and_final_answer", rendered);
    });
}

#[test]
fn sanitizer_borrows_clean_text_and_removes_control_sequences() {
    for (text, expected) in [
        ("clean\ttext\n", "clean\ttext\n"),
        ("\x07before", "before"),
        ("before\x07", "before"),
        ("\x1b[31mbefore", "before"),
        ("before\x1b[31m", "before"),
        ("before\x1b[31", "before"),
        ("\x07[31m", "[31m"),
        ("\x07", ""),
    ] {
        assert_matches!(
            sanitize_user_text(text.into()),
            Cow::Borrowed(sanitized) => assert_eq!(sanitized, expected)
        );
    }
    assert_matches!(
        sanitize_user_text("before\x1b[31mafter\x07".into()),
        Cow::Owned(sanitized) => assert_eq!(sanitized, "beforeafter")
    );
    assert_eq!(sanitize_user_text("é\u{85}中".into()), "é中");
    assert_eq!(sanitize_user_text("before\x1bafter".into()), "beforeafter");
}

#[test]
fn sanitizer_preserves_owned_buffer_for_clean_and_edge_trimmed_text() {
    for (text, expected) in [
        ("clean\ttext\n", "clean\ttext\n"),
        ("\x07before", "before"),
        ("before\x07", "before"),
        ("\x07before\x07", "before"),
        ("\x1b[31mbefore", "before"),
        ("before\x1b[31m", "before"),
        ("\x07", ""),
    ] {
        let owned = text.to_string();
        let original_pointer = owned.as_ptr();
        let original_capacity = owned.capacity();

        assert_matches!(sanitize_user_text(owned.into()), Cow::Owned(sanitized) => {
            assert_eq!(sanitized, expected);
            assert_eq!(sanitized.as_ptr(), original_pointer);
            assert_eq!(sanitized.capacity(), original_capacity);
        })
    }
}

#[test]
fn sanitizer_preallocates_owned_multi_fragment_text() {
    let text = "before\x1b[31mafter\x07".to_string();
    let original_length = text.len();

    assert_matches!(sanitize_user_text(text.into()), Cow::Owned(sanitized) => {
        assert_eq!(sanitized, "beforeafter");
        assert!(sanitized.capacity() >= original_length, "{} >= {}", sanitized.capacity(), original_length);
    })
}

#[test]
fn user_messages_have_a_visible_label_and_rail_without_changing_raw_text() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let message = "  hello from voice";
        let spoken = new_spoken_user_prompt(message.to_string());
        let typed = new_user_prompt(message.to_string(), Vec::new(), Vec::new(), Vec::new());
        let marker = spoken
            .display_hyperlink_lines(/*width*/ 40)
            .into_iter()
            .flat_map(|line| line.line.spans)
            .find(|span| span.content == "│ ")
            .expect("spoken user rail");

        assert!(
            typed
                .display_lines(/*width*/ 40)
                .iter()
                .any(|line| line.to_string().trim_end() == "YOU")
        );

        assert!(
            spoken
                .display_lines(/*width*/ 40)
                .iter()
                .any(|line| line.to_string().trim_end() == "│ hello from voice")
        );
        assert!(
            typed
                .display_lines(/*width*/ 40)
                .iter()
                .any(|line| { line.to_string().trim_end() == "│   hello from voice" })
        );
        assert!(typed.background_style().is_some());
        assert_eq!(marker.style.fg, Some(Color::Red));
        assert!(marker.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(spoken.raw_lines(), vec![Line::from(message)]);
        assert_eq!(
            spoken.display_lines_for_mode(/*width*/ 40, HistoryRenderMode::Raw),
            vec![Line::from(message)]
        );
        assert!(typed.display_lines(/*width*/ 40).iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content == "│ " && span.style.fg != Some(Color::Red))
        }));

        let area = Rect::new(
            /*x*/ 0, /*y*/ 0, /*width*/ 40, /*height*/ 3,
        );
        let mut buf = Buffer::empty(area);
        Paragraph::new(spoken.display_lines(area.width)).render(area, &mut buf);
        insta::assert_snapshot!("spoken_user_prompt", format!("{buf:?}"));
    });
}

fn replace_cached_lines(
    cell: &AgentMarkdownCell,
    update_key: impl FnOnce(&mut MarkdownRenderCacheKey),
) {
    let rendered_lines = cell
        .rendered_lines
        .as_ref()
        .expect("ordinary markdown should be cacheable");
    let mut rendered_lines = rendered_lines.cached.lock().expect("render cache lock");
    let (key, lines) = rendered_lines
        .as_mut()
        .expect("render cache should be populated");
    *lines = vec![HyperlinkLine::from("cached")];
    update_key(key);
}

#[test]
fn finalized_markdown_reuses_lines_primed_by_transcript_height() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cell = AgentMarkdownCell::new("finalized **markdown**".to_string(), Path::new("/tmp"));
        let width = 48;

        assert_eq!(cell.desired_transcript_height(width), 5);
        replace_cached_lines(&cell, |_| {});

        assert_eq!(
            visible_lines(cell.transcript_hyperlink_lines(width)),
            vec![Line::from("cached")]
        );
    });
}

#[test]
fn finalized_assistant_file_citation_renders_as_local_path_snapshot() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let cwd = std::env::temp_dir();
        let output = cwd.join("Quarterly Report.xlsx").display().to_string();
        let cell = AgentMarkdownCell::new(
            format!(
                r#"Generated :codex-file-citation{{artifact_kind="workbook" path="{output}" purpose="output"}}."#
            ),
            &cwd,
        );

        let rendered = ratatui::text::Text::from(cell.display_lines(/*width*/ 80));

        insta::assert_snapshot!(rendered, @"
───────────────────────────────────────────────────────────────────────────────
CODEX

  Generated Quarterly Report.xlsx.
───────────────────────────────────────────────────────────────────────────────");
    });
}

#[test]
fn finalized_markdown_cache_misses_when_width_or_render_style_changes() {
    let cell = AgentMarkdownCell::new("finalized **markdown**".to_string(), Path::new("/tmp"));
    let width = 48;
    let expected = cell.display_lines(width);

    replace_cached_lines(&cell, |key| key.width = key.width.saturating_sub(1));
    assert_eq!(cell.display_lines(width), expected);

    replace_cached_lines(&cell, |key| {
        key.syntax_theme_revision = key.syntax_theme_revision.wrapping_sub(1);
    });
    assert_eq!(cell.display_lines(width), expected);

    replace_cached_lines(&cell, |key| {
        key.terminal_fg = key
            .terminal_fg
            .map_or(Some((1, 2, 3)), |(r, g, b)| Some((r ^ 1, g, b)));
    });
    assert_eq!(cell.display_lines(width), expected);
}

#[test]
fn raw_markdown_bypasses_the_rich_render_cache() {
    let source = "finalized **markdown**";
    let cell = AgentMarkdownCell::new(source.to_string(), Path::new("/tmp"));
    let width = 48;

    cell.display_lines(width);
    replace_cached_lines(&cell, |_| {});

    assert_eq!(
        cell.display_lines_for_mode(width, HistoryRenderMode::Raw),
        vec![Line::from(source)]
    );
}

#[test]
fn visualization_directives_are_not_cached() {
    for markdown in [
        "::codex-inline-vis{file=\"chart.html\"}",
        "\u{e200}visualize\u{e202}{\"path\":\"/tmp/chart.html\"}\u{e201}",
    ] {
        let cell = AgentMarkdownCell::new(markdown.to_string(), Path::new("/tmp"));

        cell.display_lines(/*width*/ 48);

        assert!(cell.rendered_lines.is_none());
    }
}

#[test]
fn spoken_artifacts_link_only_real_workspace_files_and_preserve_existing_urls() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let workspace = tempfile::tempdir().expect("workspace");
        let source_directory = workspace.path().join("src");
        std::fs::create_dir(&source_directory).expect("source directory");
        let source_file = source_directory.join("lib.rs");
        std::fs::write(&source_file, "fn example() {}\n").expect("source file");
        let markdown = "中 src/lib.rs:42 and https://example.com";
        let spoken = AgentMarkdownCell::new_spoken(markdown.to_string(), workspace.path());
        let lines = spoken.display_hyperlink_lines(/*width*/ 90);
        let line = &lines[0];

        let expected_destination =
            url::Url::from_file_path(source_file.canonicalize().expect("canonical source file"))
                .expect("workspace file URL")
                .to_string();
        assert_eq!(
            line.hyperlinks
                .iter()
                .map(|link| (link.columns.clone(), link.destination.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (5..18, expected_destination.as_str()),
                (23..42, "https://example.com"),
            ]
        );
        assert!(line.line.spans.iter().any(|span| {
            span.content == "src/lib.rs:42"
                && span.style.add_modifier.contains(Modifier::UNDERLINED)
        }));
        assert_eq!(spoken.raw_lines(), vec![Line::from(markdown)]);
        insta::assert_snapshot!(
            format!(
                "{}\n{:?} -> <workspace>/src/lib.rs\n{:?} -> https://example.com",
                line.line, line.hyperlinks[0].columns, line.hyperlinks[1].columns
            ),
            @r"
    • 中 src/lib.rs:42 and https://example.com
    5..18 -> <workspace>/src/lib.rs
    23..42 -> https://example.com
    "
        );

        let ordinary = AgentMarkdownCell::new(markdown.to_string(), workspace.path());
        assert_eq!(
            ordinary.display_hyperlink_lines(/*width*/ 90)[3].hyperlinks[0].destination,
            "https://example.com"
        );
        let bare = AgentMarkdownCell::new_spoken("src/lib.rs".to_string(), workspace.path());
        assert_eq!(
            bare.display_hyperlink_lines(/*width*/ 40)[0]
                .hyperlinks
                .len(),
            1
        );
    });
}

#[test]
fn spoken_windows_relative_paths_keep_original_text_and_link_the_workspace_file() {
    let workspace = tempfile::tempdir().expect("workspace");
    let source_directory = workspace.path().join("src");
    std::fs::create_dir(&source_directory).expect("source directory");
    let source_file = source_directory.join("lib.rs");
    std::fs::write(&source_file, "fn example() {}\n").expect("source file");
    let markdown = "Updated src\\lib.rs:42";
    let spoken = AgentMarkdownCell::new_spoken(markdown.to_string(), workspace.path());
    let lines = spoken.display_hyperlink_lines(/*width*/ 80);
    let line = &lines[0];

    assert_eq!(spoken.raw_lines(), vec![Line::from(markdown)]);
    assert_eq!(line.hyperlinks.len(), 1);
    assert_eq!(line.hyperlinks[0].columns, 10..23);
    assert_eq!(
        line.hyperlinks[0].destination,
        url::Url::from_file_path(source_file.canonicalize().expect("canonical source file"))
            .expect("workspace file URL")
            .to_string()
    );
    assert!(line.line.spans.iter().any(|span| {
        span.content == "src\\lib.rs:42" && span.style.add_modifier.contains(Modifier::UNDERLINED)
    }));
    insta::assert_snapshot!(line.line.to_string(), @"• Updated src\\lib.rs:42");
}
