//! Shared history-cell building blocks reused across transcript concerns.

use super::*;

#[derive(Debug)]
pub(crate) struct PlainHistoryCell {
    pub(super) lines: Vec<Line<'static>>,
}

impl PlainHistoryCell {
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl HistoryCell for PlainHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.lines.clone())
    }
}

/// Marks any completed history cell as Codex-initiated tool activity while preserving its
/// specialized rendering (including terminal hyperlinks).
#[derive(Debug)]
pub(crate) struct CodexToolCallHistoryCell {
    inner: Box<dyn HistoryCell>,
}

impl CodexToolCallHistoryCell {
    pub(crate) fn new(inner: impl HistoryCell + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl HistoryCell for CodexToolCallHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        prepend_codex_tool_call_label(self.inner.display_lines(width.saturating_sub(2)))
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        prepend_codex_tool_call_label(self.inner.transcript_lines(width.saturating_sub(2)))
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        prepend_codex_tool_call_hyperlink_label(
            self.inner.display_hyperlink_lines(width.saturating_sub(2)),
        )
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        prepend_codex_tool_call_hyperlink_label(
            self.inner
                .transcript_hyperlink_lines(width.saturating_sub(2)),
        )
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(prepend_codex_tool_call_label(self.inner.raw_lines()))
    }

    fn is_codex_tool_call(&self) -> bool {
        true
    }

    fn has_stable_transcript_height(&self) -> bool {
        self.inner.has_stable_transcript_height()
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        self.inner.transcript_animation_tick()
    }
}

#[derive(Debug)]
pub(crate) struct WebHyperlinkHistoryCell {
    lines: Vec<HyperlinkLine>,
}

impl WebHyperlinkHistoryCell {
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self {
            lines: crate::terminal_hyperlinks::annotate_web_urls(lines),
        }
    }

    pub(crate) fn new_hyperlink_lines(lines: Vec<HyperlinkLine>) -> Self {
        Self { lines }
    }
}

impl HistoryCell for WebHyperlinkHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.lines.iter().map(|line| line.line.clone()).collect()
    }

    fn display_hyperlink_lines(&self, _width: u16) -> Vec<HyperlinkLine> {
        self.lines.clone()
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.lines.iter().map(|line| line.line.clone()))
    }
}
#[derive(Debug)]
pub(crate) struct PrefixedWrappedHistoryCell {
    text: Text<'static>,
    initial_prefix: Line<'static>,
    subsequent_prefix: Line<'static>,
}

impl PrefixedWrappedHistoryCell {
    pub(crate) fn new(
        text: impl Into<Text<'static>>,
        initial_prefix: impl Into<Line<'static>>,
        subsequent_prefix: impl Into<Line<'static>>,
    ) -> Self {
        Self {
            text: text.into(),
            initial_prefix: initial_prefix.into(),
            subsequent_prefix: subsequent_prefix.into(),
        }
    }
}

impl HistoryCell for PrefixedWrappedHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        if width == 0 {
            return Vec::new();
        }
        let opts = RtOptions::new(width.max(1) as usize)
            .initial_indent(self.initial_prefix.clone())
            .subsequent_indent(self.subsequent_prefix.clone());
        adaptive_wrap_lines(&self.text, opts)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.text.clone().lines)
    }
}

#[derive(Debug)]
pub(crate) struct ToolCallContinuationCell {
    inner: Box<dyn HistoryCell>,
}

impl ToolCallContinuationCell {
    pub(crate) fn new(inner: Box<dyn HistoryCell>) -> Self {
        Self { inner }
    }

    fn without_group_heading(mut lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
        if lines
            .first()
            .is_some_and(|line| line.to_string().is_empty())
            && lines
                .get(1)
                .is_some_and(|line| line.to_string() == "CODEX · Tool Calls")
        {
            lines.drain(..2);
        }
        lines
    }

    fn without_hyperlink_group_heading(mut lines: Vec<HyperlinkLine>) -> Vec<HyperlinkLine> {
        if lines
            .first()
            .is_some_and(|line| line.line.to_string().is_empty())
            && lines
                .get(1)
                .is_some_and(|line| line.line.to_string() == "CODEX · Tool Calls")
        {
            lines.drain(..2);
        }
        lines
    }
}

impl HistoryCell for ToolCallContinuationCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = Self::without_group_heading(self.inner.display_lines(width));
        lines.insert(0, Line::default());
        lines
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = Self::without_group_heading(self.inner.transcript_lines(width));
        lines.insert(0, Line::default());
        lines
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut lines =
            Self::without_hyperlink_group_heading(self.inner.display_hyperlink_lines(width));
        lines.insert(0, HyperlinkLine::from(""));
        lines
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut lines =
            Self::without_hyperlink_group_heading(self.inner.transcript_hyperlink_lines(width));
        lines.insert(0, HyperlinkLine::from(""));
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        Self::without_group_heading(self.inner.raw_lines())
    }

    fn is_codex_tool_call(&self) -> bool {
        true
    }

    fn is_stream_continuation(&self) -> bool {
        true
    }

    fn has_stable_transcript_height(&self) -> bool {
        self.inner.has_stable_transcript_height()
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        self.inner.transcript_animation_tick()
    }
}
#[derive(Debug)]
pub(crate) struct CompositeHistoryCell {
    pub(super) parts: Vec<Box<dyn HistoryCell>>,
}

impl CompositeHistoryCell {
    pub(crate) fn new(parts: Vec<Box<dyn HistoryCell>>) -> Self {
        Self { parts }
    }
}

impl HistoryCell for CompositeHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut out: Vec<Line<'static>> = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.display_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(Line::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut out = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.display_hyperlink_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(HyperlinkLine::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut out = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.transcript_hyperlink_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(HyperlinkLine::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut out: Vec<Line<'static>> = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.raw_lines();
            if !lines.is_empty() {
                if !first {
                    out.push(Line::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn has_stable_transcript_height(&self) -> bool {
        false
    }
}
