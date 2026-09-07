//! State owned by a retained Qt Quick label, independent of the native Qt ABI.
use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};

pub const ADAPTER_ID: &str = "windows.qt.quick-text";
pub const MAX_TEXT_UNITS: usize = 16_384;

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

/// A host write is authoritative even when it happens to equal our last translation.
/// Callers must not hold their state lock while calling Qt setters/signals.
#[derive(Clone, Debug)]
pub struct LabelText {
    source: String,
    written: Option<String>,
}
impl LabelText {
    pub fn new(source: String) -> Self {
        Self {
            source,
            written: None,
        }
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn host_write(&mut self, source: String) {
        self.source = source;
        self.written = None;
    }
    pub fn observe(&mut self, current: &str) {
        if self.written.as_deref() != Some(current) && self.source != current {
            self.host_write(current.to_owned());
        }
    }
    pub fn mark_written(&mut self, value: String) {
        self.written = Some(value);
    }
    pub fn clear_written(&mut self) {
        self.written = None;
    }
    pub fn restore(&self, current: &str) -> Option<&str> {
        (self.written.as_deref() == Some(current)).then_some(self.source.as_str())
    }
}

/// Only plain text and conservative AutoText labels are eligible. Input/Edit objects
/// are excluded by the native type gate before this content gate is consulted.
pub fn eligible_text(source: &str, format: i32) -> bool {
    !source.trim().is_empty()
        && source.encode_utf16().count() <= MAX_TEXT_UNITS
        && matches!(format, 0 | 2)
        && (format == 0 || !source.contains('<'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_changes_restore_the_source() {
        let mut label = LabelText::new("File".into());
        label.mark_written("文件".into());
        label.observe("文件");
        label.mark_written("文件菜单".into());
        assert_eq!(label.restore("文件菜单"), Some("File"));
    }
    #[test]
    fn host_update_wins_over_old_snapshot() {
        let mut label = LabelText::new("Old".into());
        label.mark_written("旧".into());
        label.host_write("New".into());
        label.mark_written("新".into());
        assert_eq!(label.restore("新"), Some("New"));
    }
    #[test]
    fn host_may_intentionally_write_the_same_translation() {
        let mut label = LabelText::new("File".into());
        label.mark_written("文件".into());
        label.host_write("文件".into());
        assert_eq!(label.restore("文件"), None);
        assert_eq!(label.source(), "文件");
    }
    #[test]
    fn unrelated_host_change_is_never_overwritten_on_stop() {
        let mut label = LabelText::new("File".into());
        label.mark_written("文件".into());
        assert_eq!(label.restore("New"), None);
        label.observe("New");
        assert_eq!(label.source(), "New");
    }
    #[test]
    fn complex_or_unbounded_text_is_rejected() {
        assert!(eligible_text("Open...", 2));
        assert!(eligible_text("A < B", 0));
        assert!(!eligible_text("<b>Open</b>", 2));
        assert!(!eligible_text("Open", 1));
        assert!(!eligible_text("Open", 4));
        assert!(!eligible_text(" ", 0));
        assert!(!eligible_text(&"x".repeat(MAX_TEXT_UNITS + 1), 0));
    }
}
