//! DEFLATE decoder (RFC 1951) with the zlib wrapper (RFC 1950), written
//! in-tree so the crate can *read* `/FlateDecode` streams of reference PDFs
//! (pdfTeX output) without a dependency. This crate never compresses its own
//! output; decoding exists for the comparison tooling and its tests.
//!
//! Bounded: the caller states the maximum decoded size and decoding stops
//! with an error past it, so a hostile stream cannot allocate without limit.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    Truncated,
    BadHeader(&'static str),
    BadBlock(&'static str),
    OutputLimit(usize),
    Adler32Mismatch,
}

impl std::fmt::Display for InflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InflateError::Truncated => write!(f, "deflate stream truncated"),
            InflateError::BadHeader(m) => write!(f, "bad zlib header: {m}"),
            InflateError::BadBlock(m) => write!(f, "bad deflate block: {m}"),
            InflateError::OutputLimit(n) => write!(f, "decoded output exceeds {n} bytes"),
            InflateError::Adler32Mismatch => write!(f, "zlib Adler-32 mismatch"),
        }
    }
}

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
    nbits: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Bits {
            data,
            pos: 0,
            bit: 0,
            nbits: 0,
        }
    }

    fn need(&mut self, n: u32) -> Result<(), InflateError> {
        while self.nbits < n {
            let b = *self.data.get(self.pos).ok_or(InflateError::Truncated)?;
            self.pos += 1;
            self.bit |= (b as u32) << self.nbits;
            self.nbits += 8;
        }
        Ok(())
    }

    fn bits(&mut self, n: u32) -> Result<u32, InflateError> {
        if n == 0 {
            return Ok(0);
        }
        self.need(n)?;
        let v = self.bit & ((1u32 << n) - 1);
        self.bit >>= n;
        self.nbits -= n;
        Ok(v)
    }

    fn align(&mut self) {
        self.bit = 0;
        self.nbits = 0;
    }
}

/// Canonical Huffman decoding table: counts per length and symbols sorted by code.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut left: i32 = 1;
        for &c in &counts[1..] {
            left = (left << 1) - c as i32;
            if left < 0 {
                return Err(InflateError::BadBlock("over-subscribed code lengths"));
            }
        }
        let mut offsets = [0u16; 16];
        for i in 1..15 {
            offsets[i + 1] = offsets[i] + counts[i];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offsets[l as usize] as usize] = sym as u16;
                offsets[l as usize] += 1;
            }
        }
        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, bits: &mut Bits<'_>) -> Result<u16, InflateError> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for len in 1..16 {
            code |= bits.bits(1)? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(InflateError::BadBlock("invalid Huffman code"))
    }
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// Decodes a raw DEFLATE stream (no zlib wrapper).
pub fn inflate_raw(data: &[u8], max_out: usize) -> Result<Vec<u8>, InflateError> {
    let mut bits = Bits::new(data);
    let mut out = Vec::new();
    loop {
        let last = bits.bits(1)? == 1;
        match bits.bits(2)? {
            0 => {
                bits.align();
                let len = u16::from_le_bytes(
                    data.get(bits.pos..bits.pos + 2)
                        .ok_or(InflateError::Truncated)?
                        .try_into()
                        .expect("2 bytes"),
                ) as usize;
                let nlen = u16::from_le_bytes(
                    data.get(bits.pos + 2..bits.pos + 4)
                        .ok_or(InflateError::Truncated)?
                        .try_into()
                        .expect("2 bytes"),
                ) as usize;
                if len != (!nlen & 0xFFFF) {
                    return Err(InflateError::BadBlock("stored length check"));
                }
                bits.pos += 4;
                let chunk = data
                    .get(bits.pos..bits.pos + len)
                    .ok_or(InflateError::Truncated)?;
                if out.len() + len > max_out {
                    return Err(InflateError::OutputLimit(max_out));
                }
                out.extend_from_slice(chunk);
                bits.pos += len;
            }
            1 => {
                let mut lengths = [0u8; 288];
                lengths[..144].fill(8);
                lengths[144..256].fill(9);
                lengths[256..280].fill(7);
                lengths[280..].fill(8);
                let lit = Huffman::new(&lengths)?;
                let dist = Huffman::new(&[5u8; 30])?;
                inflate_block(&mut bits, &mut out, &lit, &dist, max_out)?;
            }
            2 => {
                let hlit = bits.bits(5)? as usize + 257;
                let hdist = bits.bits(5)? as usize + 1;
                let hclen = bits.bits(4)? as usize + 4;
                const ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let mut cl = [0u8; 19];
                for &i in ORDER.iter().take(hclen) {
                    cl[i] = bits.bits(3)? as u8;
                }
                let clh = Huffman::new(&cl)?;
                let mut lengths = vec![0u8; hlit + hdist];
                let mut i = 0;
                while i < hlit + hdist {
                    let sym = clh.decode(&mut bits)?;
                    match sym {
                        0..=15 => {
                            lengths[i] = sym as u8;
                            i += 1;
                        }
                        16 => {
                            if i == 0 {
                                return Err(InflateError::BadBlock("repeat with no previous"));
                            }
                            let prev = lengths[i - 1];
                            let n = 3 + bits.bits(2)? as usize;
                            if i + n > lengths.len() {
                                return Err(InflateError::BadBlock("repeat overflow"));
                            }
                            lengths[i..i + n].fill(prev);
                            i += n;
                        }
                        17 => {
                            let n = 3 + bits.bits(3)? as usize;
                            if i + n > lengths.len() {
                                return Err(InflateError::BadBlock("zero-run overflow"));
                            }
                            i += n;
                        }
                        _ => {
                            let n = 11 + bits.bits(7)? as usize;
                            if i + n > lengths.len() {
                                return Err(InflateError::BadBlock("zero-run overflow"));
                            }
                            i += n;
                        }
                    }
                }
                if lengths[256] == 0 {
                    return Err(InflateError::BadBlock("no end-of-block code"));
                }
                let lit = Huffman::new(&lengths[..hlit])?;
                let dist = Huffman::new(&lengths[hlit..])?;
                inflate_block(&mut bits, &mut out, &lit, &dist, max_out)?;
            }
            _ => return Err(InflateError::BadBlock("reserved block type")),
        }
        if last {
            return Ok(out);
        }
    }
}

fn inflate_block(
    bits: &mut Bits<'_>,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
    max_out: usize,
) -> Result<(), InflateError> {
    loop {
        let sym = lit.decode(bits)? as usize;
        if sym < 256 {
            if out.len() >= max_out {
                return Err(InflateError::OutputLimit(max_out));
            }
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let i = sym - 257;
            if i >= 29 {
                return Err(InflateError::BadBlock("invalid length symbol"));
            }
            let len = LENGTH_BASE[i] as usize + bits.bits(LENGTH_EXTRA[i] as u32)? as usize;
            let d = dist.decode(bits)? as usize;
            if d >= 30 {
                return Err(InflateError::BadBlock("invalid distance symbol"));
            }
            let distance = DIST_BASE[d] as usize + bits.bits(DIST_EXTRA[d] as u32)? as usize;
            if distance > out.len() {
                return Err(InflateError::BadBlock("distance before start of output"));
            }
            if out.len() + len > max_out {
                return Err(InflateError::OutputLimit(max_out));
            }
            let start = out.len() - distance;
            for k in 0..len {
                let b = out[start + k];
                out.push(b);
            }
        }
    }
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// Decodes a zlib-wrapped stream (what `/FlateDecode` carries), verifying the
/// Adler-32 trailer when it is present.
pub fn inflate_zlib(data: &[u8], max_out: usize) -> Result<Vec<u8>, InflateError> {
    if data.len() < 2 {
        return Err(InflateError::Truncated);
    }
    let cmf = data[0];
    let flg = data[1];
    if cmf & 0x0F != 8 {
        return Err(InflateError::BadHeader("compression method is not deflate"));
    }
    if !(cmf as u16 * 256 + flg as u16).is_multiple_of(31) {
        return Err(InflateError::BadHeader("FCHECK failed"));
    }
    if flg & 0x20 != 0 {
        return Err(InflateError::BadHeader("preset dictionary not supported"));
    }
    let out = inflate_raw(&data[2..], max_out)?;
    // The trailer follows the last block; find how many bytes the decoder
    // consumed by re-running is wasteful, so recompute from the end instead:
    // pdfTeX streams end exactly with the 4-byte Adler-32.
    if data.len() >= 6 {
        let tail = &data[data.len() - 4..];
        let expected = u32::from_be_bytes([tail[0], tail[1], tail[2], tail[3]]);
        if expected != adler32(&out) {
            return Err(InflateError::Adler32Mismatch);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_block_round_trips() {
        // zlib header 78 01, stored block: BFINAL=1 BTYPE=00, LEN=5, NLEN
        let payload = b"hello";
        let mut z = vec![0x78, 0x01, 0x01, 5, 0, 0xFA, 0xFF];
        z.extend_from_slice(payload);
        z.extend_from_slice(&adler32(payload).to_be_bytes());
        assert_eq!(inflate_zlib(&z, 1024).unwrap(), payload);
        assert_eq!(
            inflate_zlib(&z, 3),
            Err(InflateError::OutputLimit(3)),
            "output bound is enforced"
        );
    }

    #[test]
    fn fixed_huffman_vector() {
        // zlib.compress(b"hello hello hello hello", 9) from CPython 3.12 (fixed block).
        let z = [
            0x78, 0xda, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x27, 0x01, 0x68, 0x03,
            0x08, 0xb1,
        ];
        assert_eq!(inflate_zlib(&z, 1024).unwrap(), b"hello hello hello hello");
        assert_eq!(inflate_zlib(&z, 10), Err(InflateError::OutputLimit(10)));
    }

    #[test]
    fn dynamic_huffman_vector() {
        // zlib.compress(TEXT, 9) from CPython 3.12, where TEXT is a pdfTeX-style
        // content line repeated three times followed by 300 short letter runs;
        // the block type is 2 (dynamic Huffman).
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.extend_from_slice(
                b"BT /F44 11.9552 Tf 72 708.045 Td [(Hello)-250(w)10(orld.)-310(This)-250(is)]TJ ET\n",
            );
        }
        for i in 0..300u32 {
            let c = b'a' + ((i * 7) % 26) as u8;
            for _ in 0..(i % 5) + 1 {
                expected.push(c);
            }
        }
        assert_eq!(expected.len(), 1146);
        let z = [
            0x78, 0xda, 0xed, 0xd1, 0xbb, 0x6e, 0x84, 0x30, 0x14, 0x04, 0xd0, 0x3e, 0x5f, 0xe1,
            0x12, 0x8a, 0x25, 0x40, 0x40, 0x9b, 0xb4, 0x2b, 0x25, 0x8a, 0xb6, 0x76, 0xb7, 0x4a,
            0xc1, 0xfb, 0x65, 0xd6, 0x80, 0x61, 0x79, 0x7c, 0x7d, 0xee, 0x9d, 0xf8, 0x27, 0xa2,
            0x64, 0x1a, 0x5b, 0xb2, 0x74, 0x34, 0x23, 0x5f, 0xa4, 0x78, 0xfe, 0x88, 0x22, 0x11,
            0x04, 0xde, 0x5b, 0x1c, 0x87, 0x42, 0x96, 0xe2, 0x1c, 0x8a, 0xb3, 0xff, 0xea, 0xf9,
            0x51, 0x2c, 0x64, 0x2e, 0x6e, 0xce, 0x67, 0xa1, 0x94, 0x76, 0x4f, 0x61, 0xec, 0x3b,
            0xab, 0x1b, 0xf8, 0x8e, 0x9e, 0x54, 0xee, 0xb9, 0xa7, 0x17, 0xba, 0xca, 0xba, 0x31,
            0x3f, 0x2f, 0x74, 0x7e, 0xc9, 0xab, 0x78, 0x97, 0x4f, 0x97, 0x5f, 0x20, 0x26, 0x75,
            0xad, 0xb5, 0x7e, 0x50, 0x32, 0x4e, 0x3b, 0x8e, 0xdb, 0xb6, 0x15, 0x14, 0xc5, 0x31,
            0xc7, 0x51, 0x55, 0xd5, 0x9d, 0xb2, 0x70, 0xd2, 0xa6, 0x19, 0x86, 0x61, 0xa5, 0xe4,
            0x9c, 0x6e, 0x9a, 0xf6, 0x7d, 0x2f, 0x29, 0x3d, 0x67, 0x4e, 0x08, 0x63, 0x0d, 0xdc,
            0x23, 0x6b, 0x09, 0x63, 0x0d, 0x5c, 0xa1, 0x0c, 0x61, 0xac, 0x81, 0xbb, 0x2f, 0x29,
            0x61, 0xac, 0x81, 0x5b, 0xf3, 0x8e, 0x30, 0xd6, 0xc0, 0x95, 0xfd, 0x4c, 0x18, 0x6b,
            0xe0, 0x34, 0x75, 0x6b, 0xa1, 0x81, 0xdb, 0xa8, 0x9b, 0x81, 0x06, 0xae, 0xa2, 0x6e,
            0x29, 0x34, 0x70, 0x03, 0x75, 0xeb, 0xa0, 0x81, 0xdb, 0xa9, 0xdb, 0x0c, 0x0d, 0x5c,
            0xad, 0x31, 0xb4, 0xb5, 0xdc, 0xb8, 0x61, 0xa8, 0xb1, 0xdc, 0x51, 0x61, 0x68, 0x6a,
            0xb9, 0x66, 0xc0, 0xd0, 0xce, 0x72, 0xd3, 0x8e, 0xa1, 0xb3, 0xe5, 0x92, 0x1a, 0x43,
            0x33, 0xcb, 0xb5, 0x23, 0x86, 0x2a, 0xcb, 0x99, 0x03, 0x43, 0x17, 0xcb, 0xa5, 0x0d,
            0x86, 0xe6, 0x96, 0xeb, 0x26, 0x0c, 0xed, 0x2d, 0x37, 0xff, 0xff, 0xc2, 0x5f, 0xfe,
            0x85, 0x6f, 0x24, 0x39, 0xbd, 0x49,
        ];
        assert_eq!((z[2] >> 1) & 3, 2, "vector is a dynamic-Huffman block");
        assert_eq!(inflate_zlib(&z, 4096).unwrap(), expected);
        assert!(matches!(
            inflate_zlib(&z[..100], 4096),
            Err(InflateError::Truncated)
        ));
    }
}
