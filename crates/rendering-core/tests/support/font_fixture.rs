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

/// Original tiny sfnt/CFF fixture with a triangular original GID1, named space
/// by the default CFF charset. Tests bind TeX code65 explicitly, never via Unicode.
#[allow(dead_code)]
pub fn cff_fixture() -> Vec<u8> {
    let source = shaping_fixture();
    let count = u16::from_be_bytes(source[4..6].try_into().unwrap()) as usize;
    let mut tables = BTreeMap::new();
    for i in 0..count {
        let at = 12 + 16 * i;
        let tag: [u8; 4] = source[at..at + 4].try_into().unwrap();
        if tag == *b"glyf" || tag == *b"loca" {
            continue;
        }
        let offset = u32::from_be_bytes(source[at + 8..at + 12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(source[at + 12..at + 16].try_into().unwrap()) as usize;
        tables.insert(tag, source[offset..offset + len].to_vec());
    }
    be16(tables.get_mut(b"maxp").unwrap(), 4, 2);
    let mut cff = vec![
        1, 0, 4, 4, 0, 1, 1, 1, 2, b'F', 0, 1, 1, 1, 3, 160, 17, 0, 0, 0, 0,
    ];
    cff.extend([
        0, 2, 1, 1, 2, 12, 14, 139, 139, 21, 239, 139, 5, 39, 239, 5, 14,
    ]);
    tables.insert(*b"CFF ", cff);
    let mut bytes = vec![0; 12 + tables.len() * 16];
    bytes[..4].copy_from_slice(b"OTTO");
    be16(&mut bytes, 4, tables.len() as u16);
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0)
        }
        let offset = bytes.len();
        let at = 12 + i * 16;
        bytes[at..at + 4].copy_from_slice(&tag);
        be32(&mut bytes, at + 8, offset as u32);
        be32(&mut bytes, at + 12, data.len() as u32);
        bytes.extend(data);
    }
    bytes
}

#[allow(dead_code)]
pub fn math_fixture(axis: i16) -> Vec<u8> {
    let original = shaping_fixture();
    let count = u16::from_be_bytes(original[4..6].try_into().unwrap()) as usize;
    let end = 12 + count * 16;
    let mut bytes = original[..end].to_vec();
    bytes.extend([0; 16]);
    bytes.extend_from_slice(&original[end..]);
    be16(&mut bytes, 4, (count + 1) as u16);
    for i in 0..count {
        let at = 12 + i * 16 + 8;
        let old = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
        be32(&mut bytes, at, old + 16);
    }
    let mut math = vec![0; 260];
    be32(&mut math, 0, 0x00010000);
    be16(&mut math, 4, 10);
    be16(&mut math, 6, 224);
    be16(&mut math, 10, 80);
    be16(&mut math, 12, 60);
    be16(&mut math, 14, u16::MAX);
    be16(&mut math, 16, 90);
    be16(&mut math, 22, axis as u16);
    be16(&mut math, 222, 75);
    be16(&mut math, 224, 8);
    be16(&mut math, 226, 22);
    for (offset, value) in [(232, 123), (246, 321)] {
        be16(&mut math, offset, 8);
        be16(&mut math, offset + 2, 1);
        be16(&mut math, offset + 4, value);
        be16(&mut math, offset + 8, 1);
        be16(&mut math, offset + 10, 1);
        be16(&mut math, offset + 12, 1);
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0)
    }
    let offset = bytes.len();
    bytes.extend(&math);
    bytes[end..end + 4].copy_from_slice(b"MATH");
    be32(&mut bytes, end + 8, offset as u32);
    be32(&mut bytes, end + 12, math.len() as u32);
    bytes
}

#[allow(dead_code)]
pub fn math_assembly_fixture(horizontal: bool) -> Vec<u8> {
    let mut b = math_fixture(0);
    let count = u16::from_be_bytes(b[4..6].try_into().unwrap()) as usize;
    let record = 12 + (count - 1) * 16;
    let offset = u32::from_be_bytes(b[record + 8..record + 12].try_into().unwrap()) as usize;
    let mut v = vec![0; 56];
    // One coverage and shared construction; original GID1 triangle only.
    for (at, n) in [
        (0, 5),
        (if horizontal { 4 } else { 2 }, 12),
        (if horizontal { 8 } else { 6 }, 1),
        (10, 18),
        (12, 1),
        (14, 1),
        (16, 1),
        (18, 12),
        (20, 2),
        (22, 1),
        (24, 100),
        (26, 1),
        (28, 200),
        (30, 0),
        (34, 2),
        (36, 1),
        (38, 0),
        (40, 20),
        (42, 100),
        (46, 1),
        (48, 20),
        (50, 20),
        (52, 100),
        (54, 1),
    ] {
        be16(&mut v, at, n)
    }
    be16(&mut b, offset + 8, 260);
    b.extend(v);
    be32(&mut b, record + 12, 316);
    b
}

#[allow(dead_code)]
pub fn math_kern_fixture() -> Vec<u8> {
    let mut b = math_fixture(0);
    let count = u16::from_be_bytes(b[4..6].try_into().unwrap()) as usize;
    let record = 12 + (count - 1) * 16;
    let offset = u32::from_be_bytes(b[record + 8..record + 12].try_into().unwrap()) as usize;
    be16(&mut b, offset + 230, 36);
    let mut k = vec![0; 40];
    for (at, n) in [
        (0, 12),
        (2, 1),
        (4, 18),
        (12, 1),
        (14, 1),
        (16, 1),
        (18, 2),
        (20, 65526),
        (24, 20),
        (28, 65533),
        (32, 5),
        (36, 8),
    ] {
        be16(&mut k, at, n)
    }
    b.extend(k);
    be32(&mut b, record + 12, 300);
    b
}
