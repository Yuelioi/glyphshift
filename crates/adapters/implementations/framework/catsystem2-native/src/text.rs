//! Human text and the single supported engine terminator stay separate. Raw
//! owner snapshots still include the marker for exact conflict checks/restore.
pub const MARKER: &[u8] = &[255, 255];

pub fn decode(bytes: &[u8]) -> Option<(&str, &[u8])> {
    let mut body = bytes;
    while let Some(prefix) = body.strip_suffix(MARKER) {
        body = prefix;
    }
    let marker = &bytes[body.len()..];
    Some((std::str::from_utf8(body).ok()?, marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separates_only_the_exact_terminal_marker() {
        let mut input = "Synthetic dialogue 对话".as_bytes().to_vec();
        assert_eq!(decode(&input), Some(("Synthetic dialogue 对话", &[][..])));
        input.extend_from_slice(MARKER);
        assert_eq!(decode(&input), Some(("Synthetic dialogue 对话", MARKER)));
        input.extend_from_slice(MARKER);
        assert_eq!(
            decode(&input),
            Some(("Synthetic dialogue 对话", &[255, 255, 255, 255][..]))
        );
        for invalid in [
            b"text\xff".as_slice(),
            b"text\xff\xff\xff",
            b"text\xff\xff\xff\xff\xff",
            b"text\xff\xff\xff\xffmore",
            b"\xc0\x80",
        ] {
            assert!(decode(invalid).is_none());
        }
    }
}
