//! Upstream-baseline harness for the `UiProfile` toggle.
//!
//! The toggle's core guarantee is that with [`UiProfile::Upstream`] the TUI renders
//! byte-identically to upstream Codex, with the `codex-dev` customizations appearing only
//! under [`UiProfile::CodexDev`]. These tests are the *mechanical* gate for that guarantee:
//! each drives the profile through [`with_test_ui_profile`] (never ambient process state)
//! and asserts a property verified against `upstream/main`, so a regression that reintroduces
//! a customization into the Upstream path fails here instead of shipping silently.
//!
//! Unlike snapshot tests that assert the *current* rendered output, these assert *upstream
//! truth*, so they cannot be satisfied by a leak. They intentionally live in one module so
//! the whole gate is reviewable in one place.
//!
//! Coverage is deliberately extensible — add one test per gated surface. Remaining surfaces
//! to cover (from the review punch-list), each already gated in source: markdown h4-h6 +
//! code-block background, user-message cell background, working-status shimmer, footer draft
//! row, MCP detail/error dim, exec transcript dim strip. Fixtures for those live in their
//! respective `*_tests.rs`; mirror the `with_test_ui_profile(Upstream)` pattern below.

use crate::history_cell::CodexToolCallHistoryCell;
use crate::history_cell::HistoryCell;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::new_active_mcp_tool_call;
use crate::markdown_render::render_markdown_text;
use crate::ui_profile::UiProfile;
use crate::ui_profile::with_test_ui_profile;
use pretty_assertions::assert_eq;
use ratatui::text::Line;

fn rendered(lines: &[Line<'static>]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

/// Under Upstream, the tool-call wrapper is a pass-through: it adds no `CODEX · Tool Calls`
/// label, no `┊` gutter, and no width reduction (upstream has neither the wrapper type nor
/// the 2-column gutter). Under CodexDev it adds the label and gutter.
#[test]
fn upstream_tool_call_wrapper_is_a_passthrough() {
    let inner_text = "a representative tool-call output line";
    let plain = PlainHistoryCell::new(vec![Line::from(inner_text)]);
    let wrapped =
        CodexToolCallHistoryCell::new(PlainHistoryCell::new(vec![Line::from(inner_text)]));

    let upstream_plain =
        with_test_ui_profile(UiProfile::Upstream, || rendered(&plain.display_lines(80)));
    let upstream_wrapped =
        with_test_ui_profile(UiProfile::Upstream, || rendered(&wrapped.display_lines(80)));

    assert_eq!(upstream_wrapped, upstream_plain);
    assert!(
        !upstream_wrapped.iter().any(|line| line.contains('┊')),
        "upstream leaked the codex-dev tool-call gutter: {upstream_wrapped:?}"
    );
    assert!(
        !upstream_wrapped
            .iter()
            .any(|line| line.contains("CODEX · Tool Calls")),
        "upstream leaked the codex-dev tool-call label: {upstream_wrapped:?}"
    );

    let codex_dev_wrapped =
        with_test_ui_profile(UiProfile::CodexDev, || rendered(&wrapped.display_lines(80)));
    assert!(
        codex_dev_wrapped
            .iter()
            .any(|line| line.contains("CODEX · Tool Calls")),
        "codex-dev lost its tool-call label: {codex_dev_wrapped:?}"
    );
    assert!(
        codex_dev_wrapped.iter().any(|line| line.contains('┊')),
        "codex-dev lost its tool-call gutter: {codex_dev_wrapped:?}"
    );
}

/// Under Upstream, a completed successful MCP call renders upstream's normal multi-line
/// output — never the codex-dev compact `· Ctrl+T details` one-liner (upstream has no such
/// collapse). Under CodexDev it renders the compact summary. This is the surface that
/// regressed twice during review, so it is pinned explicitly.
#[test]
fn upstream_completed_mcp_call_has_no_compact_summary() {
    use crate::history_cell::McpInvocation;
    use codex_protocol::mcp::CallToolResult;
    use serde_json::json;
    use std::time::Duration;

    let completed_mcp = || {
        let mut cell = new_active_mcp_tool_call(
            "call-1".to_string(),
            McpInvocation {
                server: "node_repl".to_string(),
                tool: "js".to_string(),
                arguments: Some(json!({ "code": "await tab.snapshot()" })),
            },
            /*animations_enabled*/ false,
        );
        cell.complete(
            Duration::ZERO,
            Ok(CallToolResult {
                content: vec![json!({ "type": "text", "text": "done" })],
                structured_content: None,
                is_error: None,
                meta: None,
            }),
        );
        cell
    };

    let upstream = with_test_ui_profile(UiProfile::Upstream, || {
        rendered(&completed_mcp().display_lines(40))
    });
    assert!(
        !upstream.iter().any(|line| line.contains("Ctrl+T details")),
        "upstream leaked the codex-dev compact MCP summary: {upstream:?}"
    );
    assert!(
        !upstream.iter().any(|line| line.contains('┊')),
        "upstream leaked the codex-dev MCP gutter: {upstream:?}"
    );

    let codex_dev = with_test_ui_profile(UiProfile::CodexDev, || {
        rendered(&completed_mcp().display_lines(40))
    });
    assert!(
        codex_dev.iter().any(|line| line.contains("Ctrl+T details")),
        "codex-dev lost its compact MCP summary: {codex_dev:?}"
    );
}

/// Markdown is shared by every transcript surface, so the baseline gate checks both the
/// heading modifier and fenced-code background rather than relying on a snapshot generated by
/// this fork.
#[test]
fn upstream_markdown_keeps_upstream_heading_and_code_styles() {
    let render = || render_markdown_text("#### Heading\n\n```rust\nlet answer = 42;\n```");
    let upstream = with_test_ui_profile(UiProfile::Upstream, render);
    let heading = upstream
        .lines
        .iter()
        .find(|line| line.to_string().contains("Heading"))
        .expect("heading line");
    assert!(heading.spans.iter().all(|span| {
        !span
            .style
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    }));
    let code = upstream
        .lines
        .iter()
        .find(|line| line.to_string().contains("answer"))
        .expect("code line");
    assert!(code.spans.iter().all(|span| span.style.bg.is_none()));

    let codex_dev = with_test_ui_profile(UiProfile::CodexDev, render);
    let dev_heading = codex_dev
        .lines
        .iter()
        .find(|line| line.to_string().contains("Heading"))
        .expect("codex-dev heading line");
    assert!(dev_heading.spans.iter().any(|span| {
        span.style
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    }));
}

/// Under Upstream, a user message renders as plain upstream text with no codex-dev band:
/// no `YOU` label and no `│` rail. Under CodexDev it renders the band.
#[test]
fn upstream_user_cell_has_no_you_band() {
    use crate::history_cell::UserHistoryCell;

    let user_cell = || UserHistoryCell {
        message: "hello world".into(),
        text_elements: Vec::new(),
        local_image_paths: Vec::new(),
        remote_image_urls: Vec::new(),
        spoken: false,
    };

    let upstream = with_test_ui_profile(UiProfile::Upstream, || {
        rendered(&user_cell().display_lines(40))
    });
    assert!(
        !upstream.iter().any(|line| line.contains("YOU")),
        "upstream leaked the codex-dev YOU label: {upstream:?}"
    );
    assert!(
        !upstream.iter().any(|line| line.contains('│')),
        "upstream leaked the codex-dev user rail: {upstream:?}"
    );

    let codex_dev = with_test_ui_profile(UiProfile::CodexDev, || {
        rendered(&user_cell().display_lines(40))
    });
    assert!(
        codex_dev.iter().any(|line| line.contains("YOU")),
        "codex-dev lost its YOU band: {codex_dev:?}"
    );
}
