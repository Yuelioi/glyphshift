//! Inspect a locally supplied mapped PE snapshot; never attach or mutate a process.
use glyphshift_adapter_catsystem2_native::shape::Image;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: inspect_image <mapped-image> <hex-base>"
    );
    let bytes = std::fs::read(&args[1]).unwrap();
    let base = usize::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap();
    let word = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
    let short = |at| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()) as usize;
    let pe = word(60);
    let table = pe + 24 + short(pe + 20);
    let executable = (0..short(pe + 6))
        .filter_map(|i| {
            let at = table + i * 40;
            (word(at + 36) & 0x20000000 != 0)
                .then(|| base + word(at + 12)..base + word(at + 12) + word(at + 8))
        })
        .collect();
    let image = Image {
        base,
        bytes,
        executable,
    };
    let shape = image
        .discover()
        .expect("unsupported or ambiguous object shape");
    println!("{shape:#x?}");
}
