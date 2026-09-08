//! Offline query-shape inspection. Input is a little-endian method count,
//! followed by (byte count, code bytes) for each method. This never loads or
//! executes the inspected code and does not certify an interface's provenance.
use glyphshift_adapter_vgui_localize_native::{panel_abi, refresh_abi};
use std::io::{self, Read};

fn number(input: &mut impl Read) -> io::Result<usize> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes) as usize)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: panel_shape <methods-file>")?;
    let mut input = std::fs::File::open(path)?;
    let count = number(&mut input)?;
    if count > 256 {
        return Err("too many methods".into());
    }
    let mut methods = Vec::new();
    for _ in 0..count {
        let count = number(&mut input)?;
        if count > 4096 {
            return Err("method too large".into());
        }
        let mut method = vec![0; count];
        input.read_exact(&mut method)?;
        methods.push(method);
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing input".into());
    }
    println!("Boundary: {:?}", panel_abi::identify(&methods));
    println!("Handles: {:?}", refresh_abi::handles(&methods));
    println!("Image: {:?}", refresh_abi::image(&methods));
    if let Some(member) = std::env::args().nth(2) {
        let v = std::env::args()
            .skip(3)
            .map(|s| s.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;
        if v.len() != 3 {
            return Err(
                "label inspection requires text offset, symbol offset, and setter slot".into(),
            );
        }
        println!(
            "Label: {:?}",
            refresh_abi::label_setter(
                &methods,
                member.parse()?,
                refresh_abi::ImageShape {
                    text: v[0],
                    symbol: v[1],
                    setter: v[2]
                }
            )
        );
    }
    Ok(())
}
