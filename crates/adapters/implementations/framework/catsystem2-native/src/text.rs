//! Human text and the single supported engine terminator stay separate. Raw
//! owner snapshots still include the marker for exact conflict checks/restore.
pub const MARKER: &[u8] = &[255, 255];

/// Only leading color commands are replayable without moving control codes
/// across translated text. Unknown and embedded commands remain unsupported.
pub fn classic_parts(bytes: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let mut start = 0;
    loop {
        let rest = &bytes[start..];
        let digits = if rest.starts_with(b"\\cd0x") {
            5
        } else if rest.starts_with(b"\\c0x") {
            4
        } else {
            break;
        };
        if rest
            .get(digits..digits + 8)
            .is_none_or(|s| !s.iter().all(u8::is_ascii_hexdigit))
            || rest.get(digits + 8) != Some(&b';')
        {
            break;
        }
        start += digits + 9;
    }
    let mut end = bytes.len();
    while end > start && bytes[end - 1] == 255 {
        end -= 1;
    }
    (&bytes[..start], &bytes[start..end], &bytes[end..])
}

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
    fn classic_color_prefix_and_single_byte_markers_are_exact() {
        let input = b"\\cd0xffffe2b0;\\c0xff123456;Synthetic\xff\xff";
        assert_eq!(
            classic_parts(input),
            (&input[..27], b"Synthetic".as_slice(), &[255, 255][..])
        );
        for bad in [
            b"\\c0xBAD;text".as_slice(),
            b"text\\c0xff123456;tail",
            b"\\c0xGG123456;text",
        ] {
            assert_eq!(classic_parts(bad), (&[][..], bad, &[][..]));
        }
    }
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
