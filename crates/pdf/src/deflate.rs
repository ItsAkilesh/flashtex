//! A small, deterministic zlib compressor (RFC 1950/1951) for image data the
//! exact route has to re-encode (PNG with alpha, interlaced or 16-bit PNG
//! split into colour and `/SMask`).
//!
//! One final block with the fixed Huffman code and a hash-chain LZ77
//! matcher (32 KiB window, bounded chain length, greedy). The output is not
//! byte-identical to zlib's (pdfTeX links zlib; the image *samples* are what
//! must match, not the compressed bytes) but it is a pure function of the
//! input, so exports stay byte-for-byte reproducible. `crate::inflate`
//! round-trips it in the tests.

const WINDOW: usize = 1 << 15;
const MASK: usize = WINDOW - 1;
const HASH_SIZE: usize = 1 << 15;
const MAX_CHAIN: usize = 128;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const NONE: usize = usize::MAX;

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

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    /// Appends `count` bits of `value`, least significant first.
    fn bits(&mut self, value: u32, count: u32) {
        self.acc |= u64::from(value) << self.n;
        self.n += count;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// Appends a Huffman code (most significant bit first).
    fn code(&mut self, code: u32, len: u32) {
        let mut rev = 0;
        for i in 0..len {
            rev |= ((code >> i) & 1) << (len - 1 - i);
        }
        self.bits(rev, len);
    }

    fn flush(&mut self) {
        if self.n > 0 {
            self.out.push(self.acc as u8);
            self.acc = 0;
            self.n = 0;
        }
    }

    /// Fixed literal/length code for symbol 0..=287.
    fn literal(&mut self, sym: u16) {
        match sym {
            0..=143 => self.code(0x30 + u32::from(sym), 8),
            144..=255 => self.code(0x190 + u32::from(sym - 144), 9),
            256..=279 => self.code(u32::from(sym - 256), 7),
            _ => self.code(0xC0 + u32::from(sym - 280), 8),
        }
    }

    fn matched(&mut self, len: usize, dist: usize) {
        let li = LENGTH_BASE
            .iter()
            .rposition(|&b| usize::from(b) <= len)
            .expect("len >= 3");
        self.literal(257 + li as u16);
        self.bits(
            (len - usize::from(LENGTH_BASE[li])) as u32,
            u32::from(LENGTH_EXTRA[li]),
        );
        let di = DIST_BASE
            .iter()
            .rposition(|&b| usize::from(b) <= dist)
            .expect("dist >= 1");
        self.code(di as u32, 5);
        self.bits(
            (dist - usize::from(DIST_BASE[di])) as u32,
            u32::from(DIST_EXTRA[di]),
        );
    }
}

fn hash(d: &[u8], i: usize) -> usize {
    ((usize::from(d[i]) << 10) ^ (usize::from(d[i + 1]) << 5) ^ usize::from(d[i + 2]))
        & (HASH_SIZE - 1)
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += u32::from(x);
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// Compresses `data` into a zlib stream (`/FlateDecode` payload).
pub fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut w = BitWriter {
        out: vec![0x78, 0xDA],
        acc: 0,
        n: 0,
    };
    w.bits(1, 1); // BFINAL
    w.bits(1, 2); // BTYPE = 01, fixed Huffman
    let n = data.len();
    let mut head = vec![NONE; HASH_SIZE];
    let mut prev = vec![NONE; WINDOW];
    let insert = |head: &mut Vec<usize>, prev: &mut Vec<usize>, j: usize| {
        if j + MIN_MATCH <= n {
            let h = hash(data, j);
            prev[j & MASK] = head[h];
            head[h] = j;
        }
    };
    let mut i = 0;
    while i < n {
        let (mut best_len, mut best_dist) = (0, 0);
        if i + MIN_MATCH <= n {
            let max = (n - i).min(MAX_MATCH);
            let mut cand = head[hash(data, i)];
            let mut chain = 0;
            while cand != NONE && cand < i && i - cand <= WINDOW && chain < MAX_CHAIN {
                let mut l = 0;
                while l < max && data[cand + l] == data[i + l] {
                    l += 1;
                }
                if l > best_len {
                    best_len = l;
                    best_dist = i - cand;
                    if l == max {
                        break;
                    }
                }
                let next = prev[cand & MASK];
                if next == NONE || next >= cand {
                    break;
                }
                cand = next;
                chain += 1;
            }
        }
        if best_len >= MIN_MATCH {
            w.matched(best_len, best_dist);
            for j in i..i + best_len {
                insert(&mut head, &mut prev, j);
            }
            i += best_len;
        } else {
            w.literal(u16::from(data[i]));
            insert(&mut head, &mut prev, i);
            i += 1;
        }
    }
    w.literal(256);
    w.flush();
    w.out.extend_from_slice(&adler32(data).to_be_bytes());
    w.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inflate::inflate_zlib;

    #[test]
    fn round_trips_through_inflate() {
        let mut cases: Vec<Vec<u8>> = vec![
            vec![],
            b"a".to_vec(),
            b"abcabcabcabcabcabcabcabc".to_vec(),
            vec![0; 100_000],
        ];
        let mut x: u32 = 12345;
        let noise: Vec<u8> = (0..70_000)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (x >> 16) as u8
            })
            .collect();
        cases.push(noise.clone());
        let mut mixed = noise[..40_000].to_vec();
        mixed.extend_from_slice(&noise[..40_000]);
        mixed.extend(std::iter::repeat_n(7u8, 1000));
        cases.push(mixed);
        for c in cases {
            let z = zlib_compress(&c);
            assert_eq!(inflate_zlib(&z, 1 << 24).unwrap(), c, "len {}", c.len());
            assert_eq!(zlib_compress(&c), z, "deterministic");
        }
        assert!(zlib_compress(&vec![0; 100_000]).len() < 1000);
    }
}
