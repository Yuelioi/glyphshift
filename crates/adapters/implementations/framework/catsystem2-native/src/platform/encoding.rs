//! Classic ANSI profile uses the target's active Windows code page. Reject
//! invalid sequences and default/best-fit substitution in both directions.
use windows_sys::Win32::Globalization::*;
pub fn decode(bytes: &[u8], cp: u32) -> Option<String> {
    if bytes.is_empty() {
        return Some(String::new());
    }
    let length = i32::try_from(bytes.len()).ok()?;
    unsafe {
        let size = MultiByteToWideChar(
            cp,
            MB_ERR_INVALID_CHARS,
            bytes.as_ptr(),
            length,
            std::ptr::null_mut(),
            0,
        );
        if size <= 0 {
            return None;
        }
        let mut wide = vec![0; size as usize];
        if MultiByteToWideChar(
            cp,
            MB_ERR_INVALID_CHARS,
            bytes.as_ptr(),
            length,
            wide.as_mut_ptr(),
            size,
        ) != size
        {
            return None;
        }
        let text = String::from_utf16(&wide).ok()?;
        (encode(&text, cp)?.as_slice() == bytes).then_some(text)
    }
}
pub fn encode(text: &str, cp: u32) -> Option<Vec<u8>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    let wide: Vec<_> = text.encode_utf16().collect();
    let length = i32::try_from(wide.len()).ok()?;
    // This profile describes multibyte ANSI text, not UTF-7/UTF-8 code pages.
    if !matches!(cp, 932 | 936 | 949 | 950 | 1250..=1258) {
        return None;
    }
    unsafe {
        let mut substituted = 0;
        let size = WideCharToMultiByte(
            cp,
            WC_NO_BEST_FIT_CHARS,
            wide.as_ptr(),
            length,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            &mut substituted,
        );
        if size <= 0 || substituted != 0 {
            return None;
        }
        let mut bytes = vec![0; size as usize];
        if WideCharToMultiByte(
            cp,
            WC_NO_BEST_FIT_CHARS,
            wide.as_ptr(),
            length,
            bytes.as_mut_ptr(),
            size,
            std::ptr::null(),
            &mut substituted,
        ) != size
            || substituted != 0
        {
            return None;
        }
        Some(bytes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multibyte_roundtrip_and_unrepresentable_text() {
        for (cp, text) in [
            (936, "合成对话"),
            (932, "合成テキスト"),
            (1252, "Synthetic text"),
        ] {
            assert_eq!(
                decode(&encode(text, cp).unwrap(), cp).as_deref(),
                Some(text)
            );
            assert!(encode("🦀", cp).is_none());
        }
        assert!(decode(&[0x81], 936).is_none());
        assert!(encode("合成", 1252).is_none());
    }
}
