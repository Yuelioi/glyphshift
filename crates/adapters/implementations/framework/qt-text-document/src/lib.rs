//! State and product descriptor for retained Qt `QTextDocument` text.

use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};

pub const ADAPTER_ID: &str = "windows.qt.text-document";
pub const MAX_TEXT_UNITS: usize = 16_384;

#[must_use]
pub fn descriptor() -> AdapterDescriptor {
    AdapterDescriptor::new(
        AdapterId::new(ADAPTER_ID),
        AdapterVersion::new(1, 0, 0),
        ApplyModel::RetainedObject,
        Placement::TargetProcess,
        [Feature::TextObserve, Feature::TextReplace],
    )
    .with_platforms(["windows"])
    .with_architectures(["x86_64"])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlTextTemplate {
    original: String,
    source: String,
    source_start: usize,
    source_end: usize,
}

impl HtmlTextTemplate {
    /// Accept plain `setHtml` input, conservative rich HTML with one visible literal text node,
    /// or one logical root-level inline flow. Style/script contents and outer markup are retained
    /// verbatim. A multi-run inline flow is flattened to one source string and one replacement
    /// node so translated word order stays coherent. Multiple block flows, entities, or malformed
    /// markup fail open to the host.
    #[must_use]
    pub fn parse(html: &str) -> Option<Self> {
        if html.encode_utf16().count() > MAX_TEXT_UNITS {
            return None;
        }
        let bytes = html.as_bytes();
        let mut cursor = 0_usize;
        let mut suppressed: Option<String> = None;
        let mut stack = Vec::<String>::new();
        let mut visible = Vec::<(usize, usize, String)>::new();
        let mut flow_text = String::new();
        let mut flow_start = None::<usize>;
        let mut flow_end = None::<usize>;
        let mut inline_flow_tags = 0_usize;
        let mut inline_flow_compatible = true;

        while cursor < bytes.len() {
            if bytes[cursor] == b'<' {
                let end = tag_end(bytes, cursor + 1)?;
                let tag = html.get(cursor + 1..end)?.trim();
                if tag.starts_with("!--") {
                    if suppressed.is_none() {
                        inline_flow_compatible = false;
                    }
                    let comment_end = html.get(cursor + 4..)?.find("-->")? + cursor + 4;
                    cursor = comment_end + 3;
                    continue;
                }
                if tag.starts_with('!') || tag.starts_with('?') {
                    cursor = end + 1;
                    continue;
                }
                let closing = tag.starts_with('/');
                let body = tag.trim_start_matches('/').trim_start();
                let name_end = body
                    .find(|character: char| character.is_ascii_whitespace() || character == '/')
                    .unwrap_or(body.len());
                let name = body.get(..name_end)?.to_ascii_lowercase();
                if name.is_empty() {
                    return None;
                }
                let self_closing = tag.ends_with('/')
                    || matches!(
                        name.as_str(),
                        "area"
                            | "base"
                            | "br"
                            | "col"
                            | "embed"
                            | "hr"
                            | "img"
                            | "input"
                            | "link"
                            | "meta"
                            | "param"
                            | "source"
                            | "track"
                            | "wbr"
                    );
                if !matches!(name.as_str(), "style" | "script") && suppressed.is_none() {
                    if inline_flow_tag(&name) {
                        inline_flow_tags += 1;
                        flow_start = Some(flow_start.map_or(cursor, |start| start.min(cursor)));
                        flow_end = Some(flow_end.map_or(end + 1, |finish| finish.max(end + 1)));
                    } else {
                        inline_flow_compatible = false;
                    }
                }
                if closing {
                    if stack.pop().as_deref() != Some(name.as_str()) {
                        return None;
                    }
                } else if !self_closing {
                    stack.push(name.clone());
                }
                if matches!(name.as_str(), "style" | "script") {
                    if closing {
                        if suppressed.as_deref() != Some(name.as_str()) {
                            return None;
                        }
                        suppressed = None;
                    } else if !self_closing {
                        if suppressed.is_some() {
                            return None;
                        }
                        suppressed = Some(name);
                    }
                }
                cursor = end + 1;
                continue;
            }

            let end = bytes[cursor..]
                .iter()
                .position(|byte| *byte == b'<')
                .map_or(bytes.len(), |offset| cursor + offset);
            if suppressed.is_none() {
                let text = html.get(cursor..end)?;
                flow_text.push_str(text);
                if !text.trim().is_empty() {
                    visible.push((cursor, end, text.to_owned()));
                    flow_start = Some(flow_start.map_or(cursor, |start| start.min(cursor)));
                    flow_end = Some(flow_end.map_or(end, |finish| finish.max(end)));
                }
                if text.contains('&') {
                    inline_flow_compatible = false;
                }
            }
            cursor = end;
        }
        if suppressed.is_some() {
            return None;
        }
        if !stack.is_empty() {
            return None;
        }
        if let [(source_start, source_end, source)] = visible.as_slice() {
            if source == source.trim()
                && !source.contains('&')
                && !source
                    .chars()
                    .any(|character| matches!(character, '\r' | '\n' | '\t'))
                && source.encode_utf16().count() <= MAX_TEXT_UNITS
            {
                return Some(Self {
                    original: html.to_owned(),
                    source: source.clone(),
                    source_start: *source_start,
                    source_end: *source_end,
                });
            }
        }

        if !inline_flow_compatible || inline_flow_tags == 0 || visible.is_empty() {
            return None;
        }
        let source = normalize_inline_flow_text(&flow_text)?;
        if source.encode_utf16().count() > MAX_TEXT_UNITS {
            return None;
        }
        let source_start = flow_start?;
        let source_end = flow_end?;
        Some(Self {
            original: html.to_owned(),
            source,
            source_start,
            source_end,
        })
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn original_html(&self) -> &str {
        &self.original
    }

    #[must_use]
    pub fn render(&self, replacement: &str) -> Option<String> {
        if replacement.trim().is_empty()
            || replacement.encode_utf16().count() > MAX_TEXT_UNITS
            || replacement
                .chars()
                .any(|character| matches!(character, '\r' | '\n' | '\t'))
        {
            return None;
        }
        let mut escaped = String::with_capacity(replacement.len());
        for character in replacement.chars() {
            match character {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                _ => escaped.push(character),
            }
        }
        let mut rendered = String::with_capacity(
            self.original.len() - (self.source_end - self.source_start) + escaped.len(),
        );
        rendered.push_str(self.original.get(..self.source_start)?);
        rendered.push_str(&escaped);
        rendered.push_str(self.original.get(self.source_end..)?);
        (rendered.encode_utf16().count() <= MAX_TEXT_UNITS).then_some(rendered)
    }
}

fn tag_end(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    let mut quote = None;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\'' | b'"' if quote.is_none() => quote = Some(bytes[cursor]),
            byte if quote == Some(byte) => quote = None,
            b'>' if quote.is_none() => return Some(cursor),
            _ => {}
        }
        cursor += 1;
    }
    None
}

#[derive(Clone, Debug)]
pub struct DocumentText {
    source: String,
    written: Option<String>,
}

impl DocumentText {
    #[must_use]
    pub fn new(source: String) -> Self {
        Self {
            source,
            written: None,
        }
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn host_write(&mut self, source: String) {
        self.source = source;
        self.written = None;
    }

    pub fn mark_written(&mut self, value: String) {
        self.written = Some(value);
    }

    pub fn clear_written(&mut self) {
        self.written = None;
    }

    #[must_use]
    pub fn restore(&self, current: &str) -> Option<&str> {
        (self.written.as_deref() == Some(current)).then_some(self.source.as_str())
    }
}

fn inline_flow_tag(name: &str) -> bool {
    matches!(
        name,
        "a" | "b"
            | "br"
            | "code"
            | "em"
            | "font"
            | "i"
            | "img"
            | "kbd"
            | "mark"
            | "s"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "u"
            | "wbr"
    )
}

fn normalize_inline_flow_text(text: &str) -> Option<String> {
    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;
    for character in text.chars() {
        if character.is_whitespace() {
            pending_space = !normalized.is_empty();
            continue;
        }
        if pending_space {
            normalized.push(' ');
            pending_space = false;
        }
        normalized.push(character);
    }
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_is_a_retained_x64_target_process_adapter() {
        let descriptor = descriptor();
        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::RetainedObject);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert!(descriptor.architectures().any(|arch| arch == "x86_64"));
        assert!(!descriptor.architectures().any(|arch| arch == "x86"));
    }

    #[test]
    fn plain_and_single_run_rich_html_preserve_the_host_template() {
        let plain = HtmlTextTemplate::parse("The currently active Desktop").unwrap();
        assert_eq!(plain.source(), "The currently active Desktop");
        assert_eq!(
            plain.render("当前活动桌面").as_deref(),
            Some("当前活动桌面")
        );

        let html = "<style>p { margin: 4px; }</style><p class=\"summary\">Creates a cube.</p>";
        let rich = HtmlTextTemplate::parse(html).unwrap();
        assert_eq!(rich.source(), "Creates a cube.");
        assert_eq!(
            rich.render("创建 <立方体> & 更多").as_deref(),
            Some(
                "<style>p { margin: 4px; }</style><p class=\"summary\">创建 &lt;立方体&gt; &amp; 更多</p>"
            )
        );
        assert_eq!(rich.original_html(), html);
    }

    #[test]
    fn multiline_style_with_one_visible_summary_is_supported() {
        let html = r#"<style>
p {margin-top: 4px; margin-bottom: 4px }
div.fig {margin-top: 6px; margin-bottom: 6px;}
figcaption {color: #808080}
dt, .label {font-weight: bold; margin-bottom: 0}
dd, .content {margin-left: 12px; margin-top: 0}
</style><p class="summary">Creates a tube.</p>"#;
        let template = HtmlTextTemplate::parse(html).unwrap();
        assert_eq!(template.source(), "Creates a tube.");
        assert_eq!(
            template.render("创建管体。"),
            Some(html.replace("Creates a tube.", "创建管体。"))
        );
    }

    #[test]
    fn complex_or_ambiguous_html_fails_open() {
        assert!(HtmlTextTemplate::parse("<b>Open</b>").is_some());
        assert!(HtmlTextTemplate::parse("<p>Save &amp; Close</p>").is_none());
        assert!(HtmlTextTemplate::parse("<p>Hello <b>world</b></p>").is_none());
        assert!(HtmlTextTemplate::parse("<p>one</p><p>two</p>").is_none());
        assert!(HtmlTextTemplate::parse("A < B").is_none());
        assert!(HtmlTextTemplate::parse("line one\nline two").is_none());
        assert!(HtmlTextTemplate::parse(" \t ").is_none());
        assert!(HtmlTextTemplate::parse("<style>x</style><p>text").is_none());
        assert!(HtmlTextTemplate::parse(&"x".repeat(MAX_TEXT_UNITS + 1)).is_none());
    }

    #[test]
    fn one_logical_inline_flow_is_flattened_without_fragmenting_the_source() {
        let html = concat!(
            "<style>p { margin: 4px; }</style>",
            "When <strong>safe selection</strong> is on, use the ",
            "<img src='image://example/select' width='16' height='16'> Select tool. ",
            "Hold <span class=\"keys\"><kbd>S</kbd></span> temporarily.\n",
            "This remains one logical help sentence."
        );
        let template = HtmlTextTemplate::parse(html).unwrap();
        assert_eq!(
            template.source(),
            "When safe selection is on, use the Select tool. Hold S temporarily. This remains one logical help sentence."
        );
        assert_eq!(
            template.render("启用安全选择后，请使用选择工具；按 S 可临时切换。"),
            Some(
                "<style>p { margin: 4px; }</style>启用安全选择后，请使用选择工具；按 S 可临时切换。"
                    .into()
            )
        );
    }

    #[test]
    fn restoration_only_overwrites_our_last_write() {
        let mut text = DocumentText::new("Open".into());
        text.mark_written("打开".into());
        assert_eq!(text.restore("打开"), Some("Open"));
        assert_eq!(text.restore("Application changed it"), None);

        text.host_write("New source".into());
        text.mark_written("新文本".into());
        assert_eq!(text.restore("新文本"), Some("New source"));
    }
}
