use std::fs::OpenOptions;
use std::io::Write;

const TRACE_PATH_ENV: &str = "GLYPHSHIFT_IL2CPP_TRACE_PATH";
const MAX_EVENT_BYTES: usize = 160;

pub(crate) fn trace(event: &str) {
    if event.is_empty() || event.len() > MAX_EVENT_BYTES || event.contains(['\r', '\n']) {
        return;
    }
    let Some(path) = std::env::var_os(TRACE_PATH_ENV) else {
        return;
    };
    let Ok(mut output) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(output, "{event}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_or_multiline_events_before_opening_a_path() {
        trace("");
        trace("line one\nline two");
        trace(&"x".repeat(MAX_EVENT_BYTES + 1));
    }
}
