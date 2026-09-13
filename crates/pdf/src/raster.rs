//! PNG decoding and JPEG header parsing for image XObjects, zero-dependency.
//!
//! What pdfTeX 1.40.29 (TeX Live 2026) writes, measured on the fixtures in
//! `tests/fixtures/images` (`make_oracle.py`), and what this module
//! therefore produces:
//!
//! - **PNG**: always decoded (IDAT concatenated, zlib inflated, per-row
//!   filters undone, Adam7 de-interlaced) and written as uncompressed
//!   samples for `/FlateDecode` without a predictor. Gray and RGB keep their
//!   bit depth (1, 2, 4, 8, 16); a palette becomes `/Indexed /DeviceRGB` at
//!   the index bit depth. An alpha channel (gray+alpha, RGBA) is split into
//!   the colour samples at the source depth and an 8-bit `/DeviceGray`
//!   `/SMask` (16-bit alpha reduced to its high byte); a palette with `tRNS`
//!   is expanded to 8-bit RGB plus such a mask. `gAMA`, `cHRM`, `iCCP`,
//!   `sRGB` are ignored (pdfTeX's default `\pdfimageapplygamma=0`).
//! - **JPEG**: the file bytes pass through as `/DCTDecode`; the SOF marker
//!   gives size, bit depth and component count (1 gray, 3 RGB, 4 CMYK); an
//!   Adobe APP14 CMYK file gets `/Decode [1 0 1 0 1 0 1 0]` (inverted CMYK,
//!   as Photoshop writes it).
//!
//! Anything outside that (12-bit or lossless JPEG, a `tRNS` colour key on a
//! gray or RGB PNG, CMYK JPEG without APP14 and so on) is refused with a
//! message; nothing is approximated.

use crate::inflate;

/// Largest decoded sample buffer accepted (bytes).
pub const MAX_SAMPLE_BYTES: usize = 512 * 1024 * 1024;
/// Largest image dimension accepted, in pixels.
pub const MAX_DIMENSION: u32 = 1 << 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceSpace {
    Gray,
    Rgb,
    Cmyk,
}

impl DeviceSpace {
    pub fn name(self) -> &'static str {
        match self {
            DeviceSpace::Gray => "DeviceGray",
            DeviceSpace::Rgb => "DeviceRGB",
            DeviceSpace::Cmyk => "DeviceCMYK",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Space {
    Device(DeviceSpace),
    /// `[/Indexed /DeviceRGB hival lookup]`; `lookup` is `3 × (hival + 1)` bytes.
    IndexedRgb {
        lookup: Vec<u8>,
    },
}

/// A decoded PNG ready for an image XObject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPng {
    pub width: u32,
    pub height: u32,
    pub bits: u8,
    pub space: Space,
    /// Packed samples, rows padded to a byte, no filter bytes.
    pub samples: Vec<u8>,
    /// 8-bit alpha, one byte per pixel.
    pub alpha: Option<Vec<u8>>,
    /// The mask came from an alpha channel (colour type 4 or 6), not from a
    /// palette `tRNS`. pdfTeX gives the page a transparency group only then
    /// (measured: `rgba8.png` yes, `palette8-trns.png` no).
    pub alpha_channel: bool,
    /// Chunks that were present and ignored, for the report.
    pub ignored: Vec<String>,
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn channels(color_type: u8) -> Option<usize> {
    match color_type {
        0 => Some(1),
        2 => Some(3),
        3 => Some(1),
        4 => Some(2),
        6 => Some(4),
        _ => None,
    }
}

fn row_bytes(width: usize, channels: usize, bits: usize) -> usize {
    (width * channels * bits).div_ceil(8)
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let (pa, pb, pc) = (
        (p - i16::from(a)).abs(),
        (p - i16::from(b)).abs(),
        (p - i16::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Undoes the per-row filters of one (sub)image: `data` holds `height`
/// rows of `1 + row_len` bytes; returns the rows without filter bytes and
/// the number of input bytes consumed.
fn unfilter(
    data: &[u8],
    height: usize,
    row_len: usize,
    bpp: usize,
) -> Result<(Vec<u8>, usize), String> {
    let need = height
        .checked_mul(row_len + 1)
        .ok_or("PNG: image size overflows")?;
    if data.len() < need {
        return Err(format!(
            "PNG: image data is {} bytes, {need} needed",
            data.len()
        ));
    }
    let mut out = vec![0u8; height * row_len];
    for y in 0..height {
        let src = &data[y * (row_len + 1)..(y + 1) * (row_len + 1)];
        let filter = src[0];
        let (done, rest) = out.split_at_mut(y * row_len);
        let prev: &[u8] = if y == 0 {
            &[]
        } else {
            &done[(y - 1) * row_len..]
        };
        let cur = &mut rest[..row_len];
        for x in 0..row_len {
            let raw = src[1 + x];
            let a = if x >= bpp { cur[x - bpp] } else { 0 };
            let b = if y > 0 { prev[x] } else { 0 };
            let c = if y > 0 && x >= bpp { prev[x - bpp] } else { 0 };
            cur[x] = match filter {
                0 => raw,
                1 => raw.wrapping_add(a),
                2 => raw.wrapping_add(b),
                3 => raw.wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8),
                4 => raw.wrapping_add(paeth(a, b, c)),
                f => return Err(format!("PNG: row {y} has unknown filter type {f}")),
            };
        }
    }
    Ok((out, need))
}

fn get_sample(row: &[u8], index: usize, bits: usize) -> u16 {
    match bits {
        8 => u16::from(row[index]),
        16 => u16::from_be_bytes([row[2 * index], row[2 * index + 1]]),
        _ => {
            let bit = index * bits;
            let byte = row[bit / 8];
            let shift = 8 - bits - (bit % 8);
            u16::from((byte >> shift) & ((1u8 << bits) - 1))
        }
    }
}

fn put_sample(row: &mut [u8], index: usize, bits: usize, v: u16) {
    match bits {
        8 => row[index] = v as u8,
        16 => row[2 * index..2 * index + 2].copy_from_slice(&v.to_be_bytes()),
        _ => {
            let bit = index * bits;
            let shift = 8 - bits - (bit % 8);
            let mask = ((1u8 << bits) - 1) << shift;
            row[bit / 8] = (row[bit / 8] & !mask) | (((v as u8) << shift) & mask);
        }
    }
}

/// Decodes a PNG file (see the module docs for the output shape).
pub fn decode_png(bytes: &[u8]) -> Result<DecodedPng, String> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("PNG: bad signature".into());
    }
    let mut pos = 8;
    let mut ihdr: Option<(u32, u32, u8, u8, u8)> = None;
    let mut plte: Option<Vec<u8>> = None;
    let mut trns: Option<Vec<u8>> = None;
    let mut idat = Vec::new();
    let mut ignored = Vec::new();
    let mut ended = false;
    while pos + 12 <= bytes.len() {
        let len = be32(&bytes[pos..]) as usize;
        let kind = &bytes[pos + 4..pos + 8];
        let body = bytes
            .get(pos + 8..pos + 8 + len)
            .ok_or("PNG: chunk runs past the end of the file")?;
        match kind {
            b"IHDR" => {
                if body.len() != 13 {
                    return Err("PNG: IHDR is not 13 bytes".into());
                }
                if body[10] != 0 || body[11] != 0 {
                    return Err("PNG: unknown compression or filter method".into());
                }
                ihdr = Some((be32(body), be32(&body[4..]), body[8], body[9], body[12]));
            }
            b"PLTE" => plte = Some(body.to_vec()),
            b"tRNS" => trns = Some(body.to_vec()),
            b"IDAT" => idat.extend_from_slice(body),
            b"IEND" => {
                ended = true;
                break;
            }
            other => {
                if other[0] & 0x20 == 0 {
                    return Err(format!(
                        "PNG: unknown critical chunk {}",
                        String::from_utf8_lossy(other)
                    ));
                }
                let name = String::from_utf8_lossy(other).into_owned();
                if !ignored.contains(&name) {
                    ignored.push(name);
                }
            }
        }
        pos += 12 + len;
    }
    if !ended {
        return Err("PNG: no IEND chunk".into());
    }
    let (width, height, bits, color_type, interlace) = ihdr.ok_or("PNG: no IHDR")?;
    let ch = channels(color_type).ok_or(format!("PNG: colour type {color_type} is invalid"))?;
    let valid_depth = match color_type {
        0 => matches!(bits, 1 | 2 | 4 | 8 | 16),
        3 => matches!(bits, 1 | 2 | 4 | 8),
        _ => matches!(bits, 8 | 16),
    };
    if !valid_depth {
        return Err(format!(
            "PNG: bit depth {bits} is invalid for colour type {color_type}"
        ));
    }
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(format!(
            "PNG: size {width}x{height} is outside 1..={MAX_DIMENSION}"
        ));
    }
    if interlace > 1 {
        return Err(format!("PNG: interlace method {interlace} is invalid"));
    }
    let (w, h, bitsz) = (width as usize, height as usize, bits as usize);
    let full_row = row_bytes(w, ch, bitsz);
    if full_row * h > MAX_SAMPLE_BYTES {
        return Err("PNG: decoded image is larger than 512 MiB".into());
    }
    let bpp = (ch * bitsz).div_ceil(8);
    let raw_limit = (full_row + 1) * h * 2 + 64;
    let raw = inflate::inflate_zlib(&idat, raw_limit).map_err(|e| format!("PNG: {e}"))?;
    let samples = if interlace == 0 {
        unfilter(&raw, h, full_row, bpp)?.0
    } else {
        // Adam7: decode each pass and scatter its pixels.
        const PASSES: [(usize, usize, usize, usize); 7] = [
            (0, 0, 8, 8),
            (4, 0, 8, 8),
            (0, 4, 4, 8),
            (2, 0, 4, 4),
            (0, 2, 2, 4),
            (1, 0, 2, 2),
            (0, 1, 1, 2),
        ];
        let mut out = vec![0u8; full_row * h];
        let mut offset = 0;
        for (x0, y0, dx, dy) in PASSES {
            let pw = if w > x0 { (w - x0).div_ceil(dx) } else { 0 };
            let ph = if h > y0 { (h - y0).div_ceil(dy) } else { 0 };
            if pw == 0 || ph == 0 {
                continue;
            }
            let prow = row_bytes(pw, ch, bitsz);
            let (pass, used) = unfilter(&raw[offset.min(raw.len())..], ph, prow, bpp)?;
            offset += used;
            for py in 0..ph {
                let src = &pass[py * prow..(py + 1) * prow];
                let y = y0 + py * dy;
                let dst = &mut out[y * full_row..(y + 1) * full_row];
                for px in 0..pw {
                    let x = x0 + px * dx;
                    for c in 0..ch {
                        let v = get_sample(src, px * ch + c, bitsz);
                        put_sample(dst, x * ch + c, bitsz, v);
                    }
                }
            }
        }
        out
    };

    let mut result = DecodedPng {
        width,
        height,
        bits,
        space: Space::Device(DeviceSpace::Gray),
        samples: Vec::new(),
        alpha: None,
        alpha_channel: false,
        ignored,
    };
    match color_type {
        0 | 2 => {
            if trns.is_some() {
                return Err(
                    "PNG: a tRNS colour key on a gray or RGB image is not supported by the exact route"
                        .into(),
                );
            }
            result.space = Space::Device(if color_type == 0 {
                DeviceSpace::Gray
            } else {
                DeviceSpace::Rgb
            });
            result.samples = samples;
        }
        3 => {
            let palette = plte.ok_or("PNG: palette image without PLTE")?;
            if palette.is_empty() || palette.len() % 3 != 0 || palette.len() > 768 {
                return Err(format!("PNG: PLTE is {} bytes", palette.len()));
            }
            let entries = palette.len() / 3;
            match trns {
                None => {
                    // Indices beyond the palette are an error, as in libpng.
                    for y in 0..h {
                        let row = &samples[y * full_row..(y + 1) * full_row];
                        for x in 0..w {
                            let i = get_sample(row, x, bitsz) as usize;
                            if i >= entries {
                                return Err(format!(
                                    "PNG: palette index {i} at ({x}, {y}) is outside the {entries}-entry PLTE"
                                ));
                            }
                        }
                    }
                    result.space = Space::IndexedRgb { lookup: palette };
                    result.samples = samples;
                }
                Some(t) => {
                    let mut rgb = Vec::with_capacity(w * h * 3);
                    let mut alpha = Vec::with_capacity(w * h);
                    for y in 0..h {
                        let row = &samples[y * full_row..(y + 1) * full_row];
                        for x in 0..w {
                            let i = get_sample(row, x, bitsz) as usize;
                            if i >= entries {
                                return Err(format!(
                                    "PNG: palette index {i} at ({x}, {y}) is outside the {entries}-entry PLTE"
                                ));
                            }
                            rgb.extend_from_slice(&palette[3 * i..3 * i + 3]);
                            alpha.push(t.get(i).copied().unwrap_or(255));
                        }
                    }
                    result.bits = 8;
                    result.space = Space::Device(DeviceSpace::Rgb);
                    result.samples = rgb;
                    result.alpha = Some(alpha);
                }
            }
        }
        _ => {
            // 4: gray + alpha, 6: RGB + alpha; 8 or 16 bits per sample.
            let colour = ch - 1;
            let bytes_per = bitsz / 8;
            let mut col = Vec::with_capacity(w * h * colour * bytes_per);
            let mut alpha = Vec::with_capacity(w * h);
            for px in samples.chunks_exact(ch * bytes_per) {
                col.extend_from_slice(&px[..colour * bytes_per]);
                alpha.push(px[colour * bytes_per]);
            }
            result.space = Space::Device(if color_type == 4 {
                DeviceSpace::Gray
            } else {
                DeviceSpace::Rgb
            });
            result.samples = col;
            result.alpha = Some(alpha);
            result.alpha_channel = true;
        }
    }
    Ok(result)
}

/// What a JPEG's headers say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegInfo {
    pub width: u32,
    pub height: u32,
    pub bits: u8,
    pub space: DeviceSpace,
    pub progressive: bool,
    /// Adobe APP14 marker present.
    pub adobe: bool,
    /// `/Decode [1 0 1 0 1 0 1 0]` is needed (Adobe CMYK).
    pub invert_cmyk: bool,
}

/// Reads the markers up to the first frame header.
pub fn parse_jpeg(bytes: &[u8]) -> Result<JpegInfo, String> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Err("JPEG: no SOI marker".into());
    }
    let mut pos = 2;
    let mut adobe = false;
    loop {
        // Skip fill bytes before a marker.
        while bytes.get(pos) == Some(&0xFF) && bytes.get(pos + 1) == Some(&0xFF) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&0xFF) {
            return Err(format!("JPEG: expected a marker at byte {pos}"));
        }
        let marker = *bytes
            .get(pos + 1)
            .ok_or("JPEG: truncated before a frame header")?;
        pos += 2;
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            return Err("JPEG: no frame header before the scan".into());
        }
        let len = bytes
            .get(pos..pos + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
            .ok_or("JPEG: truncated marker segment")?;
        let seg = bytes
            .get(pos + 2..pos + len)
            .filter(|_| len >= 2)
            .ok_or("JPEG: marker segment runs past the end of the file")?;
        match marker {
            0xEE if seg.starts_with(b"Adobe") => adobe = true,
            0xC0 | 0xC1 | 0xC2 => {
                if seg.len() < 6 {
                    return Err("JPEG: short frame header".into());
                }
                let bits = seg[0];
                let height = u32::from(u16::from_be_bytes([seg[1], seg[2]]));
                let width = u32::from(u16::from_be_bytes([seg[3], seg[4]]));
                let comps = seg[5];
                if bits != 8 {
                    return Err(format!(
                        "JPEG: {bits}-bit samples; PDF DCTDecode takes 8-bit JPEG only"
                    ));
                }
                if width == 0 || height == 0 {
                    return Err(
                        "JPEG: zero width or height (DNL-defined height is not supported)".into(),
                    );
                }
                let space = match comps {
                    1 => DeviceSpace::Gray,
                    3 => DeviceSpace::Rgb,
                    4 => DeviceSpace::Cmyk,
                    n => return Err(format!("JPEG: {n} components")),
                };
                if space == DeviceSpace::Cmyk && !adobe {
                    return Err(
                        "JPEG: CMYK without an Adobe APP14 marker (inversion is unknown)".into(),
                    );
                }
                return Ok(JpegInfo {
                    width,
                    height,
                    bits,
                    space,
                    progressive: marker == 0xC2,
                    adobe,
                    invert_cmyk: space == DeviceSpace::Cmyk && adobe,
                });
            }
            0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                return Err(format!(
                    "JPEG: frame type SOF{} (lossless, hierarchical or arithmetic) is not DCTDecode-compatible",
                    marker - 0xC0
                ));
            }
            _ => {}
        }
        pos += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: &[u8], body: &[u8]) -> Vec<u8> {
        let mut v = (body.len() as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(body);
        v.extend_from_slice(&[0, 0, 0, 0]); // CRC is not checked
        v
    }

    fn png(w: u32, h: u32, bits: u8, ct: u8, raw: &[u8], extra: &[Vec<u8>]) -> Vec<u8> {
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = w.to_be_bytes().to_vec();
        ihdr.extend_from_slice(&h.to_be_bytes());
        ihdr.extend_from_slice(&[bits, ct, 0, 0, 0]);
        out.extend(chunk(b"IHDR", &ihdr));
        for e in extra {
            out.extend_from_slice(e);
        }
        out.extend(chunk(b"IDAT", &crate::deflate::zlib_compress(raw)));
        out.extend(chunk(b"IEND", b""));
        out
    }

    #[test]
    fn all_five_filters_are_undone() {
        // 2x3 gray: rows use Sub, Up, Average, Paeth on known data.
        let want = [10u8, 20, 30, 40, 50, 60, 70, 80];
        let raw = [
            0, 10, 20, // None
            1, 30, 10, // Sub: 30, 40
            2, 20, 20, // Up: 50, 60
            4, 10, 10, // Paeth: a=0,b=50,c=0 -> 50+10=60? see below
        ];
        let p = png(2, 4, 8, 0, &raw, &[]);
        let d = decode_png(&p).unwrap();
        assert_eq!(&d.samples[..6], &want[..6]);
        // Paeth row 3: x0: a=0,b=50,c=0 -> pred b=50 -> 60; x1: a=60,b=60,c=50 -> p=70, pa=10,pb=10,pc=20 -> a=60 -> 70
        assert_eq!(&d.samples[6..], &[60, 70]);
        let avg = [0u8, 100, 50, 3, 10, 10];
        let d = decode_png(&png(2, 2, 8, 0, &avg, &[])).unwrap();
        // Average: x0 = 10 + (0+100)/2 = 60; x1 = 10 + (60+50)/2 = 65
        assert_eq!(d.samples, vec![100, 50, 60, 65]);
    }

    #[test]
    fn alpha_and_palette_transparency_split_into_a_mask() {
        let raw = [0u8, 1, 2, 3, 200, 4, 5, 6, 7];
        let d = decode_png(&png(2, 1, 8, 6, &raw, &[])).unwrap();
        assert_eq!(d.space, Space::Device(DeviceSpace::Rgb));
        assert_eq!(d.samples, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(d.alpha, Some(vec![200, 7]));
        let plte = chunk(b"PLTE", &[1, 2, 3, 4, 5, 6]);
        let trns = chunk(b"tRNS", &[9]);
        // 2-bit indices 1, 0 -> 0b0100_0000
        let d = decode_png(&png(2, 1, 2, 3, &[0, 0x40], &[plte.clone(), trns])).unwrap();
        assert_eq!(d.bits, 8);
        assert_eq!(d.samples, vec![4, 5, 6, 1, 2, 3]);
        assert_eq!(d.alpha, Some(vec![255, 9]));
        let d = decode_png(&png(2, 1, 2, 3, &[0, 0x40], &[plte.clone()])).unwrap();
        assert_eq!(d.bits, 2);
        assert_eq!(
            d.space,
            Space::IndexedRgb {
                lookup: vec![1, 2, 3, 4, 5, 6]
            }
        );
        let e = decode_png(&png(2, 1, 2, 3, &[0, 0x80], &[plte])).unwrap_err();
        assert!(e.contains("outside the 2-entry PLTE"), "{e}");
    }

    #[test]
    fn refusals_are_precise() {
        assert!(decode_png(b"nope").unwrap_err().contains("signature"));
        let trns = chunk(b"tRNS", &[0, 1]);
        let e = decode_png(&png(1, 1, 8, 0, &[0, 5], &[trns])).unwrap_err();
        assert!(e.contains("colour key"), "{e}");
        assert!(
            parse_jpeg(&[0xFF, 0xD8, 0xFF, 0xC3, 0, 8, 8, 0, 1, 0, 1, 1])
                .unwrap_err()
                .contains("SOF3")
        );
    }
}
