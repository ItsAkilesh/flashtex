//! Offline image degradation tool used only to build the Grok handwriting
//! corpus under `tests/grok-corpus/`. Not part of the bridge runtime; it
//! reuses the crate's existing `image` dependency instead of pulling in
//! ImageMagick or an extra Rust crate. Every transform is deterministic
//! (seeded, no wall-clock or OS randomness) so `generate.sh` reproduces
//! byte-identical corpus images across runs and machines.
//!
//! Usage: corpus_augment <op> [args..]
//!   rotate           <in> <out> <degrees>
//!   perspective      <in> <out> <strength>      (0.0-1.0 keystone amount)
//!   contrast         <in> <out> <delta>          (negative washes out)
//!   lighting-gradient <in> <out> <strength>      (0.0-1.0)
//!   jpeg             <in> <out> <quality>        (1-100, re-encodes as JPEG)
//!   downscale-blur   <in> <out> <factor>         (0.0-1.0 shrink-then-grow)
//!   noise            <in> <out> <amount> <seed>  (per-pixel +-amount)
//!   pure-noise       <out> <width> <height> <seed>
//!   solid            <out> <width> <height> <r> <g> <b>
use image::{imageops, ImageBuffer, Rgb, RgbImage};
use std::{env, io::BufWriter};

/// Small deterministic PRNG (xorshift64*) so noise is reproducible without
/// depending on the `rand` crate.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E3779B97F4A7C15)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform float in [-1.0, 1.0].
    fn next_signed(&mut self) -> f64 {
        let bits = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        bits * 2.0 - 1.0
    }
}

fn load(path: &str) -> RgbImage {
    image::open(path)
        .unwrap_or_else(|e| panic!("failed to open {path}: {e}"))
        .to_rgb8()
}

fn save(img: &RgbImage, path: &str, jpeg_quality: Option<u8>) {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        let quality = jpeg_quality.unwrap_or(92);
        let file =
            std::fs::File::create(path).unwrap_or_else(|e| panic!("failed to create {path}: {e}"));
        let mut writer = BufWriter::new(file);
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .unwrap_or_else(|e| panic!("failed to encode jpeg {path}: {e}"));
    } else {
        img.save(path)
            .unwrap_or_else(|e| panic!("failed to save {path}: {e}"));
    }
}

fn sample_bilinear(img: &RgbImage, x: f64, y: f64) -> Option<[u8; 3]> {
    let (w, h) = img.dimensions();
    if x < 0.0 || y < 0.0 || x > (w - 1) as f64 || y > (h - 1) as f64 {
        return None;
    }
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let p00 = img.get_pixel(x0, y0).0;
    let p10 = img.get_pixel(x1, y0).0;
    let p01 = img.get_pixel(x0, y1).0;
    let p11 = img.get_pixel(x1, y1).0;
    let mut out = [0u8; 3];
    for c in 0..3 {
        let top = p00[c] as f64 * (1.0 - fx) + p10[c] as f64 * fx;
        let bottom = p01[c] as f64 * (1.0 - fx) + p11[c] as f64 * fx;
        out[c] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
    }
    Some(out)
}

/// Solve for the 8 coefficients (a..h) of the projective map
/// to.x = (a*x + b*y + c) / (g*x + h*y + 1)
/// to.y = (d*x + e*y + f) / (g*x + h*y + 1)
/// from 4 point correspondences, via Gaussian elimination with partial
/// pivoting on the resulting 8x8 linear system.
fn solve_homography(from: [(f64, f64); 4], to: [(f64, f64); 4]) -> [f64; 8] {
    let mut m = [[0.0f64; 9]; 8];
    for i in 0..4 {
        let (x, y) = from[i];
        let (xp, yp) = to[i];
        m[2 * i] = [x, y, 1.0, 0.0, 0.0, 0.0, -x * xp, -y * xp, xp];
        m[2 * i + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -x * yp, -y * yp, yp];
    }
    // Gaussian elimination with partial pivoting.
    for col in 0..8 {
        let mut pivot = col;
        for row in (col + 1)..8 {
            if m[row][col].abs() > m[pivot][col].abs() {
                pivot = row;
            }
        }
        m.swap(col, pivot);
        let d = m[col][col];
        assert!(d.abs() > 1e-12, "singular homography system");
        for v in m[col][col..].iter_mut() {
            *v /= d;
        }
        let pivot_row = m[col];
        for (row, target) in m.iter_mut().enumerate() {
            if row != col {
                let factor = target[col];
                for (v, p) in target[col..].iter_mut().zip(pivot_row[col..].iter()) {
                    *v -= factor * p;
                }
            }
        }
    }
    let mut coeffs = [0.0f64; 8];
    for (i, c) in coeffs.iter_mut().enumerate() {
        *c = m[i][8];
    }
    coeffs
}

fn op_rotate(input: &str, output: &str, degrees: f64) {
    let src = load(input);
    let (w, h) = src.dimensions();
    let theta = -degrees.to_radians();
    let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
    let (sin_t, cos_t) = theta.sin_cos();
    let out = ImageBuffer::from_fn(w, h, |x, y| {
        let (dx, dy) = (x as f64 - cx, y as f64 - cy);
        let sx = cx + dx * cos_t - dy * sin_t;
        let sy = cy + dx * sin_t + dy * cos_t;
        Rgb(sample_bilinear(&src, sx, sy).unwrap_or([255, 255, 255]))
    });
    save(&out, output, None);
}

fn op_perspective(input: &str, output: &str, strength: f64) {
    let src = load(input);
    let (w, h) = src.dimensions();
    let (wf, hf) = (w as f64, h as f64);
    // Keystone: top edge pulled inward and the whole top shifted down a
    // little, as if the camera were tilted back relative to the page.
    let inset = wf * strength * 0.22;
    let drop = hf * strength * 0.10;
    let distorted = [
        (inset, drop),      // top-left
        (wf - inset, drop), // top-right
        (wf, hf),           // bottom-right
        (0.0, hf),          // bottom-left
    ];
    let source_rect = [(0.0, 0.0), (wf, 0.0), (wf, hf), (0.0, hf)];
    let coeffs = solve_homography(distorted, source_rect);
    let [a, b, c, d, e, f, g, hh] = coeffs;
    let out = ImageBuffer::from_fn(w, h, |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        let denom = g * xf + hh * yf + 1.0;
        let sx = (a * xf + b * yf + c) / denom;
        let sy = (d * xf + e * yf + f) / denom;
        Rgb(sample_bilinear(&src, sx, sy).unwrap_or([255, 255, 255]))
    });
    save(&out, output, None);
}

fn op_contrast(input: &str, output: &str, delta: f32) {
    let src = load(input);
    let out = imageops::contrast(&src, delta);
    save(&out, output, None);
}

fn op_lighting_gradient(input: &str, output: &str, strength: f64) {
    let src = load(input);
    let (w, h) = src.dimensions();
    let out = ImageBuffer::from_fn(w, h, |x, y| {
        let nx = x as f64 / (w.max(1) - 1).max(1) as f64;
        let ny = y as f64 / (h.max(1) - 1).max(1) as f64;
        // Diagonal shadow: brightest at top-left, darkest at bottom-right.
        let d = ((nx + ny) / 2.0).clamp(0.0, 1.0);
        let factor = 1.0 - strength * d;
        let p = src.get_pixel(x, y).0;
        Rgb([
            (p[0] as f64 * factor).round().clamp(0.0, 255.0) as u8,
            (p[1] as f64 * factor).round().clamp(0.0, 255.0) as u8,
            (p[2] as f64 * factor).round().clamp(0.0, 255.0) as u8,
        ])
    });
    save(&out, output, None);
}

fn op_jpeg(input: &str, output: &str, quality: u8) {
    let src = load(input);
    save(&src, output, Some(quality));
}

fn op_downscale_blur(input: &str, output: &str, factor: f64) {
    let src = load(input);
    let (w, h) = src.dimensions();
    let sw = ((w as f64 * factor).round() as u32).max(1);
    let sh = ((h as f64 * factor).round() as u32).max(1);
    let small = imageops::resize(&src, sw, sh, imageops::FilterType::Nearest);
    let back = imageops::resize(&small, w, h, imageops::FilterType::Triangle);
    save(&back, output, None);
}

fn op_noise(input: &str, output: &str, amount: f64, seed: u64) {
    let src = load(input);
    let mut rng = Rng::new(seed);
    let out = ImageBuffer::from_fn(src.width(), src.height(), |x, y| {
        let p = src.get_pixel(x, y).0;
        let mut px = [0u8; 3];
        for c in 0..3 {
            let n = rng.next_signed() * amount;
            px[c] = (p[c] as f64 + n).round().clamp(0.0, 255.0) as u8;
        }
        Rgb(px)
    });
    save(&out, output, None);
}

fn op_pure_noise(output: &str, width: u32, height: u32, seed: u64) {
    let mut rng = Rng::new(seed);
    let out = ImageBuffer::from_fn(width, height, |_, _| {
        let mut px = [0u8; 3];
        for p in px.iter_mut() {
            *p = ((rng.next_u64() >> 8) & 0xFF) as u8;
        }
        Rgb(px)
    });
    save(&out, output, None);
}

fn op_solid(output: &str, width: u32, height: u32, r: u8, g: u8, b: u8) {
    let out = ImageBuffer::from_fn(width, height, |_, _| Rgb([r, g, b]));
    save(&out, output, None);
}

const USAGE: &str = "\
corpus_augment <op> [args..]
  rotate            <in> <out> <degrees>
  perspective       <in> <out> <strength>       (0.0-1.0 keystone amount)
  contrast          <in> <out> <delta>          (negative washes out)
  lighting-gradient <in> <out> <strength>       (0.0-1.0)
  jpeg              <in> <out> <quality>        (1-100, re-encodes as JPEG)
  downscale-blur    <in> <out> <factor>         (0.0-1.0 shrink-then-grow)
  noise             <in> <out> <amount> <seed>  (per-pixel +-amount)
  pure-noise        <out> <width> <height> <seed>
  solid             <out> <width> <height> <r> <g> <b>";

fn usage() -> ! {
    eprintln!("{USAGE}");
    std::process::exit(2);
}

fn arg(args: &[String], i: usize) -> &str {
    args.get(i)
        .unwrap_or_else(|| panic!("missing argument {i}"))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    match args[1].as_str() {
        "rotate" => op_rotate(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap()),
        "perspective" => {
            op_perspective(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap())
        }
        "contrast" => op_contrast(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap()),
        "lighting-gradient" => {
            op_lighting_gradient(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap())
        }
        "jpeg" => op_jpeg(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap()),
        "downscale-blur" => {
            op_downscale_blur(arg(&args, 2), arg(&args, 3), arg(&args, 4).parse().unwrap())
        }
        "noise" => op_noise(
            arg(&args, 2),
            arg(&args, 3),
            arg(&args, 4).parse().unwrap(),
            arg(&args, 5).parse().unwrap(),
        ),
        "pure-noise" => op_pure_noise(
            arg(&args, 2),
            arg(&args, 3).parse().unwrap(),
            arg(&args, 4).parse().unwrap(),
            arg(&args, 5).parse().unwrap(),
        ),
        "solid" => op_solid(
            arg(&args, 2),
            arg(&args, 3).parse().unwrap(),
            arg(&args, 4).parse().unwrap(),
            arg(&args, 5).parse().unwrap(),
            arg(&args, 6).parse().unwrap(),
            arg(&args, 7).parse().unwrap(),
        ),
        other => {
            eprintln!("unknown op: {other}");
            usage();
        }
    }
}
