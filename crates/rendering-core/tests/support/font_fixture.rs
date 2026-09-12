use std::collections::BTreeMap;
fn be16(data: &mut [u8], at: usize, n: u16) {
    data[at..at + 2].copy_from_slice(&n.to_be_bytes());
}
fn be32(data: &mut [u8], at: usize, n: u32) {
    data[at..at + 4].copy_from_slice(&n.to_be_bytes());
}
pub fn fixture() -> Vec<u8> {
    build(false, false, false)
}
pub fn triangle_fixture() -> Vec<u8> {
    build(true, false, false)
}
#[allow(dead_code)]
pub fn grid_fixture() -> Vec<u8> {
    build(true, true, false)
}
#[allow(dead_code)]
pub fn shaping_fixture() -> Vec<u8> {
    build(true, false, true)
}
fn build(triangle: bool, grid: bool, shaping: bool) -> Vec<u8> {
    let mut tables = BTreeMap::new();
    let mut head = vec![0; 54];
    be32(&mut head, 0, 0x00010000);
    be32(&mut head, 12, 0x5f0f3cf5);
    be16(&mut head, 18, 1000);
    tables.insert(*b"head", head);
    let mut maxp = vec![0; 32];
    be32(&mut maxp, 0, 0x00010000);
    be16(&mut maxp, 4, 3);
    tables.insert(*b"maxp", maxp);
    if triangle {
        // One original three-point on-curve triangle at glyph1; no instructions.
        tables.insert(*b"loca", vec![0, 0, 0, 0, 0, 10, 0, 10]);
        tables.insert(
            *b"glyf",
            vec![
                0, 1, 0, 0, 0, 0, 0, 100, 0, 100, 0, 2, 0, 0, 0x31, 0x33, 0x27, 100, 100, 100,
            ],
        );
        if grid {
            // Glyph2 is glyph1 translated by (+50,-50) with ROUND_XY_TO_GRID.
            // Glyph1 stays original; UPEM1000 gives half-pixel ties at10ppem.
            let mut component = vec![
                0xff, 0xff, 0, 0, 0, 0, 0, 100, 0, 100, 0, 7, 0, 1, 0, 50, 0xff, 0xce,
            ];
            tables.get_mut(b"glyf").unwrap().append(&mut component);
            tables.insert(*b"loca", vec![0, 0, 0, 0, 0, 10, 0, 19]);
        }
    } else {
        tables.insert(*b"loca", vec![0; 8]);
        tables.insert(*b"glyf", vec![]);
    }
    let mut hhea = vec![0; 36];
    be32(&mut hhea, 0, 0x00010000);
    be16(&mut hhea, 34, 2);
    tables.insert(*b"hhea", hhea);
    tables.insert(*b"hmtx", vec![0; 10]);
    let mut name = vec![0; 18];
    be16(&mut name, 2, 1);
    be16(&mut name, 4, 18);
    be16(&mut name, 6, 3);
    be16(&mut name, 8, 1);
    be16(&mut name, 10, 0x0409);
    be16(&mut name, 12, 6);
    be16(&mut name, 14, 12);
    for ch in "FTTest".encode_utf16() {
        name.extend_from_slice(&ch.to_be_bytes());
    }
    tables.insert(*b"name", name);
    if shaping {
        // Original synthetic Unicode format12 map: A and é -> original GID1.
        let mut cmap = vec![0; 12 + 16 + 24];
        be16(&mut cmap, 2, 1);
        be16(&mut cmap, 4, 3);
        be16(&mut cmap, 6, 10);
        be32(&mut cmap, 8, 12);
        be16(&mut cmap, 12, 12);
        be32(&mut cmap, 16, 40);
        be32(&mut cmap, 24, 2);
        for (at, code) in [(28, 65), (40, 233)] {
            be32(&mut cmap, at, code);
            be32(&mut cmap, at + 4, code);
            be32(&mut cmap, at + 8, 1);
        }
        tables.insert(*b"cmap", cmap);
        be16(tables.get_mut(b"hmtx").unwrap(), 4, 500);
    }
    let mut bytes = vec![0; 12 + tables.len() * 16];
    be32(&mut bytes, 0, 0x00010000);
    be16(&mut bytes, 4, tables.len() as u16);
    for (index, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let start = bytes.len();
        let at = 12 + index * 16;
        bytes[at..at + 4].copy_from_slice(&tag);
        be32(&mut bytes, at + 8, start as u32);
        be32(&mut bytes, at + 12, data.len() as u32);
        bytes.extend_from_slice(&data);
    }
    bytes
}
