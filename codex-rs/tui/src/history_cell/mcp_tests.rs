use super::*;
use base64::Engine;
use codex_protocol::mcp::CallToolResult;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

#[test]
fn mcp_error_style_is_profile_specific_in_transcript() {
    let mut cell = new_active_mcp_tool_call(
        "call-id".to_string(),
        McpInvocation {
            server: "server".to_string(),
            tool: "tool".to_string(),
            arguments: None,
        },
        false,
    );
    cell.complete(Duration::ZERO, Err("boom".to_string()));
    let error_style = |profile| {
        let lines = crate::ui_profile::with_test_ui_profile(profile, || cell.transcript_lines(80));
        lines
            .iter()
            .find(|line| line.to_string().contains("Error: boom"))
            .expect("error line")
            .spans
            .iter()
            .map(|span| span.style)
            .collect::<Vec<_>>()
    };
    let upstream = error_style(crate::ui_profile::UiProfile::Upstream);
    let codex = error_style(crate::ui_profile::UiProfile::CodexDev);
    assert!(upstream.iter().all(|style| style.fg != Some(Color::Red)));
    assert!(codex.iter().any(|style| style.fg == Some(Color::Red)));
}

#[test]
fn mcp_inventory_connection_states() {
    use McpServerConnectionStatus as Status;

    let statuses = [
        ("unknown", None),
        ("starting", Some(Status::Starting)),
        ("failed", Some(Status::Failed)),
        ("disabled", Some(Status::Disabled)),
        ("deferred", Some(Status::NotStarted)),
        ("connected-empty", Some(Status::Connected)),
        ("cancelled", Some(Status::Cancelled)),
        ("auth", Some(Status::AuthenticationRequired)),
    ]
    .into_iter()
    .map(|(name, runtime_status)| McpServerStatus {
        server_capabilities: None,
        name: name.to_string(),
        runtime_status,
        plugin_id: None,
        server_info: None,
        tools: HashMap::new(),
        tools_error: None,
        resources: Vec::new(),
        resource_templates: Vec::new(),
        auth_status: McpAuthStatus::Unknown,
    })
    .collect::<Vec<_>>();
    let cell =
        new_mcp_tools_output_from_statuses(&statuses, McpServerStatusDetail::ToolsAndAuthOnly);
    let rendered = cell
        .display_lines(/*width*/ 100)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!(rendered);
    for (name, expected) in [
        ("auth", status_style(StatusTone::Attention)),
        ("cancelled", Style::default().dim()),
        ("connected-empty", status_style(StatusTone::Success)),
        ("deferred", Style::default().dim()),
        ("disabled", Style::default().dim()),
        ("failed", status_style(StatusTone::Failure)),
        ("starting", accent_style()),
        ("unknown", Style::default().dim()),
    ] {
        let row = cell
            .lines
            .iter()
            .find(|line| line.spans.get(1).is_some_and(|span| span.content == name))
            .expect("connection-state row");
        assert_eq!(
            (row.spans[0].style, row.spans[3].style),
            (expected, expected)
        );
    }
}

#[test]
fn mcp_inventory_older_app_server_authentication() {
    let statuses = serde_json::from_value::<Vec<McpServerStatus>>(json!([
        {
            "name": "legacy-auth",
            "tools": {},
            "resources": [],
            "resourceTemplates": [],
            "authStatus": "notLoggedIn"
        },
        {
            "name": "legacy-healthy",
            "tools": {"lookup": {"name": "lookup", "inputSchema": {"type": "object"}}},
            "resources": [],
            "resourceTemplates": [],
            "authStatus": "oAuth"
        },
        {
            "name": "runtime-disabled",
            "runtimeStatus": "disabled",
            "tools": {},
            "resources": [],
            "resourceTemplates": [],
            "authStatus": "notLoggedIn"
        }
    ]))
    .expect("mixed-version MCP statuses");
    let cell =
        new_mcp_tools_output_from_statuses(&statuses, McpServerStatusDetail::ToolsAndAuthOnly);
    let rendered = cell
        .display_lines(/*width*/ 100)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!(rendered);
    let styles = cell
        .lines
        .iter()
        .filter(|line| {
            line.spans
                .get(1)
                .is_some_and(|span| statuses.iter().any(|status| span.content == status.name))
        })
        .map(|line| (line.spans[0].style, line.spans[3].style))
        .collect::<Vec<_>>();
    let attention = status_style(StatusTone::Attention);
    let neutral = Style::default().dim();
    assert_eq!(
        styles,
        vec![
            (attention, attention),
            (neutral, neutral),
            (neutral, neutral)
        ]
    );
}

fn result(content: Vec<Value>) -> CallToolResult {
    CallToolResult {
        content,
        structured_content: None,
        is_error: None,
        meta: None,
    }
}

#[test]
fn code_mode_output_shares_a_row_budget_across_blocks() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let mut cell = new_active_mcp_tool_call(
            "browser-call".to_string(),
            McpInvocation {
                server: "node_repl".to_string(),
                tool: "js".to_string(),
                arguments: Some(json!({"title": "Inspect page", "code": "await tab.snapshot()"})),
            },
            /*animations_enabled*/ false,
        );
        cell.complete(
            Duration::ZERO,
            Ok(result(vec![
                json!({"type": "text", "text": "Script completed\nOutput:\n"}),
                json!({"type": "text", "text": "Page title\nNavigation\nMain content"}),
                json!({"type": "text", "text": "Button\nLink\nFooter"}),
            ])),
        );

        let display = cell
            .display_lines(/*width*/ 40)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(display, @"

    CODEX · Tool Calls
    ┊ ✓ node_repl.js · 0ms · Ctrl+T details
    ");
        let transcript = cell
            .transcript_lines(/*width*/ 100)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(transcript.contains("await tab.snapshot()"));
        assert!(transcript.ends_with("┊     Button\n┊     Link\n┊     Footer"));
    });
}

#[test]
fn code_mode_output_keeps_trailing_failure_diagnostics() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let mut cell = new_active_mcp_tool_call(
            "browser-error".to_string(),
            McpInvocation {
                server: "node_repl".to_string(),
                tool: "js".to_string(),
                arguments: Some(json!({"title": "Inspect page"})),
            },
            /*animations_enabled*/ false,
        );
        cell.complete(
        Duration::ZERO,
        Ok(CallToolResult {
            is_error: Some(true),
            ..result(vec![
                json!({"type": "text", "text": "Script failed"}),
                json!({"type": "text", "text": "Page title\nNavigation\nMain content\nButton\nLink\nFooter"}),
                json!({"type": "text", "text": "Script error:\npermission denied"}),
            ])
        }),
    );
        let display = cell
            .display_lines(/*width*/ 40)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(display, @"

    CODEX · Tool Calls
    ┊ • Inspect page
    ┊   └ Script failed
    ┊     Page title
    ┊     … more · ctrl+t
    ┊     Script error:
    ┊     permission denied
    ");
    });
}

#[test]
fn code_mode_output_row_budget_applies_after_wrapping_and_to_errors() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let output = format!("{}\ntranscript tail", "Browser 页面 👩‍💻\n".repeat(40));
        for server in ["node_repl", "cua_repl"] {
            for completion in [
                Ok(result(vec![json!({"type": "text", "text": output})])),
                Ok(result(vec![json!({
                    "type": "text",
                    "text": format!("https://example.com/{}\ntranscript tail", "页面".repeat(100)),
                })])),
                Ok(CallToolResult {
                    is_error: Some(true),
                    ..result(vec![json!({"type": "text", "text": output})])
                }),
                Err(output.clone()),
            ] {
                let mut cell = new_active_mcp_tool_call(
                    "browser-call".to_string(),
                    McpInvocation {
                        server: server.to_string(),
                        tool: "js".to_string(),
                        arguments: Some(json!({"title": "Inspect"})),
                    },
                    /*animations_enabled*/ false,
                );
                cell.complete(Duration::ZERO, completion);
                for width in [20, 40, 80] {
                    let display = cell.display_lines(width);
                    assert!(
                        display.len() == 4 || display.len() == 4 + TOOL_CALL_MAX_LINES,
                        "unexpected compact/expanded tool-call height: {display:?}"
                    );
                    assert!(
                        display
                            .iter()
                            .filter(|line| line.to_string() != "CODEX · Tool Calls")
                            .all(|line| line.width() <= usize::from(width))
                    );
                    if display.len() > 4 {
                        assert!(display[6].to_string().starts_with("┊     … more · ctrl+"));
                    }
                    let transcript = cell
                        .transcript_lines(width)
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n");
                    assert!(transcript.contains("transcript"));
                    assert!(transcript.contains("tail"));
                }
            }
        }
    });
}

#[test]
fn completed_success_is_compact_but_transcript_keeps_details() {
    let mut cell = new_active_mcp_tool_call(
        "call-compact".to_string(),
        McpInvocation {
            server: "linear".to_string(),
            tool: "get_issue".to_string(),
            arguments: Some(json!({"id": "ENG-42"})),
        },
        /* animations_enabled */ false,
    );
    cell.complete(
        Duration::from_millis(1250),
        Ok(result(vec![
            json!({"type": "text", "text": "Issue details"}),
        ])),
    );

    let display =
        crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
            cell.display_lines(/* width */ 100)
        });
    assert_eq!(
        display.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["", "CODEX · Tool Calls", "", "┊ ✓ linear.get_issue"]
    );
    let transcript =
        crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
            cell.transcript_lines(/* width */ 100)
        });
    assert_eq!(
        transcript
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![
            "",
            "CODEX · Tool Calls",
            "",
            "┊ • Called linear.get_issue({\"id\":\"ENG-42\"})",
            "┊   └ Issue details",
        ]
    );
}

#[test]
fn failed_call_keeps_error_visible_in_history() {
    let mut cell = new_active_mcp_tool_call(
        "call-failed".to_string(),
        McpInvocation {
            server: "linear".to_string(),
            tool: "get_issue".to_string(),
            arguments: None,
        },
        /* animations_enabled */ false,
    );
    cell.complete(
        Duration::from_secs(1),
        Ok(CallToolResult {
            is_error: Some(true),
            ..result(vec![json!({"type": "text", "text": "Not found"})])
        }),
    );

    let rendered = cell
        .display_lines(/* width */ 100)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(rendered.contains("Called linear.get_issue()"));
    assert!(rendered.contains("Not found"));
    assert!(!rendered.contains("Ctrl+T details"));
}

#[test]
fn projected_content_preserves_width_dependent_rendering() {
    let text = "{\"result\": [1, 2, 3], \"text\": \"long output 🦀\"}";
    let malformed = json!({"type": "image", "data": PNG});
    let invalid_metadata =
        json!({"type": "text", "text": "not a valid block", "annotations": {"priority": "high"}});
    let unknown = json!({"type": "future_block", "text": "unknown output 🦀"});
    let projected = McpToolResult::new(
        result(vec![
            json!({"type": "text", "text": text}),
            json!({"type": "image", "mimeType": "image/png", "data": PNG}),
            json!({"type": "audio", "mimeType": "audio/wav", "data": "audio data"}),
            json!({"type": "resource", "resource": {"uri": "file:///text.txt", "text": "resource body"}}),
            json!({"type": "resource", "resource": {"uri": "file:///blob.bin", "blob": "binary data"}}),
            json!({"type": "resource_link", "uri": "file:///linked.txt", "name": "linked"}),
            malformed.clone(),
            invalid_metadata.clone(),
            unknown.clone(),
        ]),
        McpResultKind::Standard,
    );

    for width in [1, 8, 40, 120, RAW_TOOL_OUTPUT_WIDTH] {
        let format_text =
            |text: &str| format_and_truncate_tool_result(text, TOOL_CALL_MAX_LINES, width);
        assert_eq!(
            projected
                .content
                .iter()
                .map(|block| block.render(width))
                .collect::<Vec<_>>(),
            vec![
                format_text(text),
                "Returned image".to_string(),
                "<audio content>".to_string(),
                "embedded resource: file:///text.txt".to_string(),
                "embedded resource: file:///blob.bin".to_string(),
                "link: file:///linked.txt".to_string(),
                format_text(&malformed.to_string()),
                format_text(&invalid_metadata.to_string()),
                format_text(&unknown.to_string()),
            ],
        );
    }
}

#[test]
fn projected_image_marker_still_requires_a_complete_image() {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(PNG)
        .expect("decode PNG fixture");
    let truncated = base64::engine::general_purpose::STANDARD.encode(&bytes[..33]);
    let invalid = json!({"type": "image", "mimeType": "image/png", "data": truncated});
    let valid = json!({"type": "image", "mimeType": "image/png", "data": format!("data:image/png;base64,{PNG}")});

    let projected = McpToolResult::new(result(vec![invalid.clone()]), McpResultKind::Standard);
    assert!(!projected.has_image);
    assert_eq!(projected.content[0].render(/*width*/ 80), "Returned image");

    let projected = McpToolResult::new(result(vec![invalid, valid]), McpResultKind::Standard);
    assert!(projected.has_image);
}

#[test]
fn code_mode_preserves_text_fields_on_nontext_and_unknown_blocks() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let mut cell = new_active_mcp_tool_call(
            "call-code-mode".to_string(),
            McpInvocation {
                server: "node_repl".to_string(),
                tool: "js".to_string(),
                arguments: Some(json!({"title": "Inspect results"})),
            },
            /*animations_enabled*/ false,
        );
        let unknown = json!({"type": "future_block", "text": "Script completed\nOutput:\nunknown-side output"});
        let tool_result = result(vec![
            json!({"type": "image", "mimeType": "image/png", "data": PNG, "text": "Script completed\nOutput:\nimage-side output"}),
            unknown.clone(),
        ]);
        cell.complete(Duration::ZERO, Ok(tool_result.clone()));

        let narrow = cell.display_lines(/*width*/ 16);
        assert!(
            narrow
                .iter()
                .filter(|line| line.to_string() != "CODEX · Tool Calls")
                .all(|line| line.width() <= 16)
        );
        assert_eq!(narrow.len(), 4);
        assert!(narrow[3].to_string().contains('✓'));

        let display = cell
            .display_lines(/*width*/ 200)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let transcript = cell
            .transcript_lines(/*width*/ 200)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(format!("history:\n{display}\n\ntranscript:\n{transcript}"), @r#"
    history:

    CODEX · Tool Calls
    ┊ ✓ node_repl.js · 0ms · Ctrl+T details

    transcript:

    CODEX · Tool Calls
    ┊ • Called node_repl.js({"title":"Inspect results"})
    ┊   └ Returned image
    ┊     Script completed
    ┊     Output:
    ┊     image-side output
    ┊     Script completed
    ┊     Output:
    ┊     unknown-side output
    "#);
        assert_eq!(
            cell.raw_lines(),
            vec![
                Line::default(),
                Line::from("CODEX · Tool Calls"),
                Line::from("┊ Called node_repl.js({\"title\":\"Inspect results\"})"),
                Line::from("┊ Returned image"),
                Line::from(format!(
                    "┊ {}",
                    format_and_truncate_tool_result(
                        &unknown.to_string(),
                        TOOL_CALL_MAX_LINES,
                        RAW_TOOL_OUTPUT_WIDTH,
                    )
                )),
            ],
        );

        let mut cua_cell = new_active_mcp_tool_call(
            "call-cua-repl".to_string(),
            McpInvocation {
                server: "cua_repl".to_string(),
                tool: "js".to_string(),
                arguments: None,
            },
            /*animations_enabled*/ false,
        );
        cua_cell.complete(Duration::ZERO, Ok(tool_result));
        let display = cua_cell
            .display_lines(/*width*/ 200)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let transcript = cua_cell
            .transcript_lines(/*width*/ 200)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(format!("history:\n{display}\n\ntranscript:\n{transcript}"), @"
    history:

    CODEX · Tool Calls
    ┊ ✓ cua_repl.js · 0ms · Ctrl+T details

    transcript:

    CODEX · Tool Calls
    ┊ • Called cua_repl.js()
    ┊   └ Returned image
    ┊     Script completed
    ┊     Output:
    ┊     image-side output
    ┊     Script completed
    ┊     Output:
    ┊     unknown-side output
    ");
    });
}

#[test]
fn titled_image_call_keeps_error_and_full_title_when_narrow() {
    crate::ui_profile::with_test_ui_profile(crate::ui_profile::UiProfile::CodexDev, || {
        let title = "Inspect a very long screenshot title 🦀";
        let mut cell = new_active_mcp_tool_call(
            "call".into(),
            McpInvocation {
                server: "node_repl".into(),
                tool: "js".into(),
                arguments: Some(json!({"title": title})),
            },
            /*animations_enabled*/ false,
        );
        assert_eq!(
            cell.display_lines(/*width*/ 80)[2].to_string(),
            format!("┊ • {title}")
        );
        cell.complete(
            Duration::ZERO,
            Ok(CallToolResult {
                is_error: Some(true),
                ..result(vec![
                    json!({"type": "text", "text": "Screenshot partially failed"}),
                    json!({"type": "image", "mimeType": "image/png", "data": PNG}),
                ])
            }),
        );
        let lines = cell.display_lines(/*width*/ 32);
        assert!(lines[2].width() <= 32);
        assert_eq!(lines[2].spans[1].style, "•".red().bold().style);
        insta::assert_snapshot!(
            lines
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            cell.raw_lines()
                .iter()
                .any(|line| line.to_string().contains(title))
        );
        assert!(
            cell.transcript_lines(/*width*/ 200)
                .iter()
                .any(|line| line.to_string().contains(title))
        );
    });
}
