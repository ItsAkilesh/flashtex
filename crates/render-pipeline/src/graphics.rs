//! `\includegraphics`: image header probing and graphicx sizing.
//!
//! Natural sizes follow pdfTeX (the oracle the fixtures are checked
//! against), never a TeX engine at run time:
//!
//! * PNG — `IHDR` pixels; resolution from `pHYs` when its unit is the metre,
//!   rounded to whole dpi as pdfTeX's `writepng.c` does; otherwise 72 dpi.
//! * JPEG — `SOFn` pixels; JFIF `APP0` density (unit 1 = dpi, 2 = dots per
//!   cm); otherwise 72 dpi.
//! * PDF — page 1 (or `page=`) CropBox (graphicx's default `pagebox`)
//!   clipped to the MediaBox, both inheritable through `/Parent`, with
//!   `/Rotate`. Only uncompressed page objects are read; a PDF whose page
//!   tree lives in compressed object streams is reported, never guessed.
//!
//! graphicx semantics (`graphicx.sty` `\Gin@esetsize`, `\Gin@ii`): keys
//! before the first `angle` request the size of the unrotated image
//! (`width`/`height` win over `scale`; both with `keepaspectratio` take the
//! smaller factor); `angle` rotates counter-clockwise about the reference
//! point and the box becomes the rotated bounding box; `width`/`height`/
//! `totalheight`/`scale` after an `angle` rescale that rotated box.

/// One TeX point in PDF big points.
pub const BP_PER_PT: f64 = 72.0 / 72.27;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Pdf,
}

impl ImageFormat {
    pub fn wire_name(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Pdf => "pdf",
        }
    }
}

/// What the header says about an image.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageInfo {
    pub format: ImageFormat,
    /// Natural size in PDF big points.
    pub width_bp: f64,
    pub height_bp: f64,
    /// Raster dimensions (PNG/JPEG).
    pub pixels: Option<(u32, u32)>,
    /// PDF only: the box's lower-left corner in the page's own space and
    /// the page's `/Rotate` (0/90/180/270), so a painter can map the page.
    pub pdf_box: Option<[f64; 4]>,
    pub pdf_rotate: i32,
    pub pdf_page: u32,
}

/// Probes `bytes` (any of the supported formats, sniffed by signature).
pub fn probe(bytes: &[u8], pdf_page: u32) -> Result<ImageInfo, String> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        probe_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        probe_jpeg(bytes)
    } else if bytes.starts_with(b"%PDF-") {
        probe_pdf(bytes, pdf_page.max(1))
    } else {
        Err("not a PNG, JPEG or PDF file (unrecognised signature)".into())
    }
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn probe_png(b: &[u8]) -> Result<ImageInfo, String> {
    let mut at = 8usize;
    let mut size = None;
    let mut dpi = None;
    while at + 12 <= b.len() {
        let len = be32(&b[at..]) as usize;
        let kind = &b[at + 4..at + 8];
        let data = b.get(at + 8..at + 8 + len).ok_or("PNG chunk runs past the end of the file")?;
        match kind {
            b"IHDR" if len >= 8 => size = Some((be32(data), be32(&data[4..]))),
            b"pHYs" if len >= 9 && data[8] == 1 => {
                // writepng.c: (int)(pixels_per_meter * 0.0254 + 0.5)
                let x = (f64::from(be32(data)) * 0.0254 + 0.5).floor();
                let y = (f64::from(be32(&data[4..])) * 0.0254 + 0.5).floor();
                if x > 0.0 && y > 0.0 {
                    dpi = Some((x, y));
                }
            }
            b"IDAT" | b"IEND" => break,
            _ => {}
        }
        at += 12 + len;
    }
    let (w, h) = size.ok_or("PNG has no IHDR chunk")?;
    if w == 0 || h == 0 {
        return Err("PNG has a zero dimension".into());
    }
    let (dx, dy) = dpi.unwrap_or((72.0, 72.0));
    Ok(ImageInfo {
        format: ImageFormat::Png,
        width_bp: f64::from(w) * 72.0 / dx,
        height_bp: f64::from(h) * 72.0 / dy,
        pixels: Some((w, h)),
        pdf_box: None,
        pdf_rotate: 0,
        pdf_page: 0,
    })
}

fn probe_jpeg(b: &[u8]) -> Result<ImageInfo, String> {
    let mut at = 2usize;
    let mut dpi = None;
    while at + 4 <= b.len() {
        if b[at] != 0xFF {
            return Err("JPEG marker stream is corrupt".into());
        }
        let marker = b[at + 1];
        if marker == 0xFF {
            at += 1;
            continue;
        }
        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) {
            at += 2;
            continue;
        }
        let len = usize::from(u16::from_be_bytes([b[at + 2], b[at + 3]]));
        let seg = b.get(at + 4..at + 2 + len).ok_or("JPEG segment runs past the end of the file")?;
        match marker {
            0xE0 if seg.len() >= 12 && seg.starts_with(b"JFIF\0") => {
                let unit = seg[7];
                let x = f64::from(u16::from_be_bytes([seg[8], seg[9]]));
                let y = f64::from(u16::from_be_bytes([seg[10], seg[11]]));
                if x > 0.0 && y > 0.0 {
                    dpi = match unit {
                        1 => Some((x, y)),
                        2 => Some((x * 2.54, y * 2.54)),
                        _ => None,
                    };
                }
            }
            0xC0..=0xCF if !matches!(marker, 0xC4 | 0xC8 | 0xCC) => {
                if seg.len() < 5 {
                    return Err("JPEG frame header is truncated".into());
                }
                let h = u32::from(u16::from_be_bytes([seg[1], seg[2]]));
                let w = u32::from(u16::from_be_bytes([seg[3], seg[4]]));
                if w == 0 || h == 0 {
                    return Err("JPEG has a zero dimension".into());
                }
                let (dx, dy) = dpi.unwrap_or((72.0, 72.0));
                return Ok(ImageInfo {
                    format: ImageFormat::Jpeg,
                    width_bp: f64::from(w) * 72.0 / dx,
                    height_bp: f64::from(h) * 72.0 / dy,
                    pixels: Some((w, h)),
                    pdf_box: None,
                    pdf_rotate: 0,
                    pdf_page: 0,
                });
            }
            _ => {}
        }
        at += 2 + len;
    }
    Err("JPEG has no frame header".into())
}

/// The dictionary text of `N 0 obj << ... >>` (uncompressed objects only).
fn pdf_object(b: &[u8], num: u32) -> Option<&[u8]> {
    let needle = format!("{num} 0 obj");
    let mut from = 0;
    while let Some(pos) = find(&b[from..], needle.as_bytes()) {
        let start = from + pos;
        let ok_before = start == 0 || !b[start - 1].is_ascii_digit();
        if ok_before {
            let body = &b[start + needle.len()..];
            let end = find(body, b"endobj").unwrap_or(body.len());
            return Some(&body[..end]);
        }
        from = start + needle.len();
    }
    None
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The value text after `/Key` in a dictionary (not in a nested stream).
fn dict_value<'a>(dict: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let k = format!("/{key}");
    let mut from = 0;
    while let Some(pos) = find(&dict[from..], k.as_bytes()) {
        let after = from + pos + k.len();
        // `/Type` must not match `/TypeX`.
        if dict.get(after).is_none_or(|c| !c.is_ascii_alphanumeric()) {
            let v = &dict[after..];
            let skip = v.iter().position(|c| !c.is_ascii_whitespace()).unwrap_or(v.len());
            return Some(&v[skip..]);
        }
        from = after;
    }
    None
}

fn ref_num(v: &[u8]) -> Option<u32> {
    let s = std::str::from_utf8(&v[..v.len().min(32)]).ok()?;
    let mut it = s.split_whitespace();
    let n = it.next()?.parse().ok()?;
    let _g: u32 = it.next()?.parse().ok()?;
    it.next()?.starts_with('R').then_some(n)
}

fn number_array(v: &[u8]) -> Option<[f64; 4]> {
    if v.first() != Some(&b'[') {
        return None;
    }
    let end = v.iter().position(|c| *c == b']')?;
    let s = std::str::from_utf8(&v[1..end]).ok()?;
    let n: Vec<f64> = s.split_whitespace().filter_map(|t| t.parse().ok()).collect();
    (n.len() == 4).then(|| [n[0].min(n[2]), n[1].min(n[3]), n[0].max(n[2]), n[1].max(n[3])])
}

fn is_type(dict: &[u8], ty: &str) -> bool {
    dict_value(dict, "Type").is_some_and(|v| v.starts_with(format!("/{ty}").as_bytes()) && v.get(ty.len() + 1).is_none_or(|c| !c.is_ascii_alphanumeric()))
}

fn probe_pdf(b: &[u8], page: u32) -> Result<ImageInfo, String> {
    let compressed = find(b, b"/ObjStm").is_some();
    let trailer_root = find(b, b"/Root").and_then(|p| ref_num(dict_value(&b[p..], "Root")?));
    let fail = |what: &str| {
        if compressed {
            format!("PDF {what}: the page tree is in a compressed object stream, which is not read yet")
        } else {
            format!("PDF {what}")
        }
    };
    let root = trailer_root.ok_or_else(|| fail("has no /Root"))?;
    let catalog = pdf_object(b, root).ok_or_else(|| fail("catalog object not found"))?;
    let pages = dict_value(catalog, "Pages").and_then(ref_num).ok_or_else(|| fail("catalog has no /Pages"))?;
    // Walk the page tree to the requested page (1-based), depth-first.
    let mut remaining = page;
    let mut node = pages;
    let mut chain: Vec<u32> = Vec::new();
    'walk: for _ in 0..64 {
        let dict = pdf_object(b, node).ok_or_else(|| fail("page tree object not found"))?;
        chain.push(node);
        if is_type(dict, "Page") {
            if remaining == 1 {
                break 'walk;
            }
            return Err(format!("PDF has fewer than {page} pages"));
        }
        let kids = dict_value(dict, "Kids").ok_or_else(|| fail("page tree node has no /Kids"))?;
        let end = kids.iter().position(|c| *c == b']').unwrap_or(kids.len());
        let text = std::str::from_utf8(&kids[1.min(end)..end]).map_err(|_| fail("has unreadable /Kids"))?;
        let toks: Vec<&str> = text.split_whitespace().collect();
        let mut next = None;
        for t in toks.chunks(3) {
            let Some(n) = t.first().and_then(|n| n.parse::<u32>().ok()) else { continue };
            let kid = pdf_object(b, n).ok_or_else(|| fail("page object not found"))?;
            let count = if is_type(kid, "Page") {
                1
            } else {
                dict_value(kid, "Count").and_then(|v| std::str::from_utf8(&v[..v.len().min(12)]).ok()?.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()).unwrap_or(1)
            };
            if remaining <= count {
                next = Some(n);
                break;
            }
            remaining -= count;
        }
        match next {
            Some(n) => node = n,
            None => return Err(format!("PDF has fewer than {page} pages")),
        }
    }
    // Inheritable attributes: nearest ancestor wins.
    let inherited = |key: &str| -> Option<&[u8]> {
        chain.iter().rev().find_map(|n| pdf_object(b, *n).and_then(|d| dict_value(d, key)))
    };
    let media = inherited("MediaBox").and_then(number_array).ok_or_else(|| fail("page has no readable /MediaBox"))?;
    let crop = inherited("CropBox").and_then(number_array).map(|c| [c[0].max(media[0]), c[1].max(media[1]), c[2].min(media[2]), c[3].min(media[3])]).unwrap_or(media);
    let rotate = inherited("Rotate")
        .and_then(|v| std::str::from_utf8(&v[..v.len().min(8)]).ok()?.split(|c: char| !(c.is_ascii_digit() || c == '-')).next()?.parse::<i32>().ok())
        .unwrap_or(0)
        .rem_euclid(360);
    let (w, h) = (crop[2] - crop[0], crop[3] - crop[1]);
    if w <= 0.0 || h <= 0.0 {
        return Err("PDF page box is empty".into());
    }
    let (w, h) = if rotate % 180 == 90 { (h, w) } else { (w, h) };
    Ok(ImageInfo {
        format: ImageFormat::Pdf,
        width_bp: w,
        height_bp: h,
        pixels: None,
        pdf_box: Some(crop),
        pdf_rotate: rotate,
        pdf_page: page,
    })
}

/// Lengths a graphicx dimension may refer to, in TeX points.
#[derive(Debug, Clone, Copy)]
pub struct LengthEnv {
    pub text_width: f64,
    pub text_height: f64,
    pub paper_width: f64,
    pub paper_height: f64,
    /// `em`/`ex` of the current font.
    pub em: f64,
    pub ex: f64,
}

/// `<factor><unit>` or `<factor>\textwidth`-style dimension, in TeX points.
pub fn parse_dimen(raw: &str, env: &LengthEnv) -> Option<f64> {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let split = s.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == ',')).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let factor = if num.is_empty() || num == "-" || num == "+" {
        if num == "-" { -1.0 } else { 1.0 }
    } else {
        num.replace(',', ".").parse::<f64>().ok()?
    };
    let per = match unit {
        "pt" => 1.0,
        "bp" => 72.27 / 72.0,
        "in" => 72.27,
        "cm" => 72.27 / 2.54,
        "mm" => 72.27 / 25.4,
        "pc" => 12.0,
        "dd" => 1238.0 / 1157.0,
        "cc" => 12.0 * 1238.0 / 1157.0,
        "sp" => 1.0 / 65536.0,
        "em" => env.em,
        "ex" => env.ex,
        "\\textwidth" | "\\linewidth" | "\\columnwidth" | "\\hsize" => env.text_width,
        "\\textheight" | "\\vsize" => env.text_height,
        "\\paperwidth" => env.paper_width,
        "\\paperheight" => env.paper_height,
        _ => return None,
    };
    Some(factor * per)
}

/// One parsed `key=value` of the optional argument, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum GKey {
    Width(f64),
    Height(f64),
    TotalHeight(f64),
    Scale(f64),
    Angle(f64),
    KeepAspectRatio(bool),
    Page(u32),
    /// `trim=<left> <bottom> <right> <top>`, TeX points (`\Gin@trim`).
    Trim([f64; 4]),
    /// `viewport=<llx> <lly> <urx> <ury>`, TeX points (`\Gin@viewport`;
    /// pdftex.def makes `bb` the same key).
    Viewport([f64; 4]),
    /// `clip` (only meaningful with a viewport; pdftex.def clips the box).
    Clip(bool),
    /// `draft`: a framed placeholder with the file name.
    Draft(bool),
    /// `origin=<letters>` for every `angle` (`\Gin@erotate` is expanded
    /// after all keys are set).
    Origin(String),
    /// Recognised but not honoured (`pagebox`, `decodearray`, ...):
    /// reported as a limitation.
    Unsupported(String),
}

/// A `\Gin@defaultbp` value in TeX points: a bare number is big points; a
/// dimension is taken as is.
fn default_bp(raw: &str, env: &LengthEnv) -> Option<f64> {
    let raw = raw.trim();
    match raw.parse::<f64>() {
        Ok(bp) => Some(bp / BP_PER_PT),
        Err(_) => parse_dimen(raw, env),
    }
}

/// The four `\Gread@parse@vp` values of `trim`/`viewport`.
fn four_bp(raw: &str, env: &LengthEnv) -> Option<[f64; 4]> {
    let v: Vec<f64> = raw.split_whitespace().map(|p| default_bp(p, env)).collect::<Option<_>>()?;
    (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
}

/// Parses the optional argument. Errors name the offending entry.
pub fn parse_keys(options: &str, env: &LengthEnv) -> (Vec<GKey>, Vec<String>) {
    let mut keys = Vec::new();
    let mut problems = Vec::new();
    for entry in split_top_level(options) {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (k, v) = match entry.split_once('=') {
            Some((k, v)) => (k.trim(), Some(v.trim().trim_matches(|c| c == '{' || c == '}'))),
            None => (entry, None),
        };
        let dimen = |v: Option<&str>| v.and_then(|v| parse_dimen(v, env));
        let number = |v: Option<&str>| v.and_then(|v| v.parse::<f64>().ok());
        let key = match k {
            "width" => dimen(v).map(GKey::Width),
            "height" => dimen(v).map(GKey::Height),
            "totalheight" => dimen(v).map(GKey::TotalHeight),
            "scale" => number(v).map(GKey::Scale),
            "angle" => number(v).map(GKey::Angle),
            "keepaspectratio" => Some(GKey::KeepAspectRatio(v.is_none_or(|v| v != "false"))),
            "page" => v.and_then(|v| v.parse().ok()).map(GKey::Page),
            "trim" => v.and_then(|v| four_bp(v, env)).map(GKey::Trim),
            "viewport" | "bb" => v.and_then(|v| four_bp(v, env)).map(GKey::Viewport),
            "clip" => Some(GKey::Clip(v.is_none_or(|v| v != "false"))),
            "draft" => Some(GKey::Draft(v.is_none_or(|v| v != "false"))),
            "origin" => Some(GKey::Origin(v.unwrap_or("c").to_string())),
            "natwidth" | "natheight" | "bbllx" | "bblly" | "bburx" | "bbury" | "hiresbb" | "pagebox" | "decodearray" | "interpolate" | "type" | "ext" | "read" | "command" => Some(GKey::Unsupported(k.to_string())),
            "alt" | "actualtext" | "artifact" | "quiet" => None,
            _ => {
                problems.push(format!("unknown \\includegraphics key '{k}'"));
                continue;
            }
        };
        match key {
            Some(key) => keys.push(key),
            None if matches!(k, "alt" | "actualtext" | "artifact" | "quiet") => {}
            None => problems.push(format!("could not read \\includegraphics key '{entry}'")),
        }
    }
    (keys, problems)
}

fn split_top_level(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// The placed box of one graphic, in TeX points, plus how the image maps
/// into it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphicBox {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    /// Affine map from the image's unit square (u right, v up, origin at
    /// the image's lower-left) to box coordinates (x right from the box's
    /// left edge, y UP from the baseline), in TeX points:
    /// `(x, y) = (e + a*u + c*v, f + b*u + d*v)`.
    pub matrix: [f64; 6],
}

/// graphicx sizing of an image whose natural size is `nat_w` x `nat_h`
/// TeX points (no viewport, clip or draft; see [`place_image`]).
pub fn size_box(nat_w: f64, nat_h: f64, keys: &[GKey]) -> GraphicBox {
    const NO_LENGTHS: LengthEnv = LengthEnv { text_width: 0.0, text_height: 0.0, paper_width: 0.0, paper_height: 0.0, em: 0.0, ex: 0.0 };
    place_image(nat_w, nat_h, keys, false, false, &NO_LENGTHS).gbox
}

/// A TeX box in points (y up from the baseline, x right from the left
/// edge) and the affine map `[a b c d e f]` of what it holds (a content
/// box's own coordinates, or an image's unit square) into it:
/// `(x, y) = (e + a*u + c*v, f + b*u + d*v)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TBox {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub matrix: [f64; 6],
}

/// graphics.sty `\Gscale@div`: `a / b`, and 1 for a zero divisor (it
/// reports "Division by 0" and divides `a` by itself).
fn scale_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        1.0
    } else {
        a / b
    }
}

impl TBox {
    /// An `\hbox` of material in its own coordinates.
    pub fn content(width: f64, height: f64, depth: f64) -> TBox {
        TBox { width, height, depth, matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] }
    }

    /// `\Gscale@box{sx}[sy]` (graphics.sty): the content drawn scaled from
    /// the reference point; height and depth swap for a negative `sy`; a
    /// negative `sx` sets the box `\hb@xt@-sx\wd{\kern-sx\wd ...\hss}`.
    pub fn scale(self, sx: f64, sy: f64) -> TBox {
        let (height, depth) = if sy < 0.0 { (-sy * self.depth, -sy * self.height) } else { (sy * self.height, sy * self.depth) };
        let dx = if sx < 0.0 { -sx * self.width } else { 0.0 };
        let m = self.matrix;
        TBox {
            width: sx.abs() * self.width,
            height,
            depth,
            matrix: [sx * m[0], sy * m[1], sx * m[2], sy * m[3], sx * m[4] + dx, sy * m[5]],
        }
    }

    /// `\Gscale@@box` (`\resizebox`, graphicx `width`/`height` after an
    /// angle): `None` is `!`. `total` measures `\totalheight` instead of
    /// `\height`; `iso` (`keepaspectratio`) takes the smaller factor.
    pub fn resize(self, width: Option<f64>, height: Option<f64>, total: bool, iso: bool) -> TBox {
        let measured = if total { self.height + self.depth } else { self.height };
        match (width, height) {
            (None, None) => self,
            (None, Some(h)) => {
                let f = scale_div(h, measured);
                self.scale(f, f)
            }
            (Some(w), None) => {
                let f = scale_div(w, self.width);
                self.scale(f, f)
            }
            (Some(w), Some(h)) => {
                let (mut a, mut b) = (scale_div(w, self.width), scale_div(h, measured));
                if iso {
                    if a > b {
                        a = b;
                    } else {
                        b = a;
                    }
                }
                self.scale(a, b)
            }
        }
    }

    /// `\Grot@box` (graphics.sty) about the point `(ox, oy)` of the box:
    /// the result is the rotated box's bounding box, the origin point keeps
    /// its height and the new left edge is the leftmost corner.
    pub fn rotate(self, degrees: f64, ox: f64, oy: f64) -> TBox {
        let (s, c) = degrees.to_radians().sin_cos();
        // trig.sty tabulates the quarter turns exactly.
        let (s, c) = if (degrees / 90.0).fract() == 0.0 { (s.round() + 0.0, c.round() + 0.0) } else { (s, c) };
        let px = |a: f64, b: f64| c * a - s * b;
        let py = |a: f64, b: f64| s * a + c * b;
        let (l, r, h, d) = (-ox, self.width - ox, self.height - oy, -self.depth - oy);
        let corners = [(l, h), (l, d), (r, h), (r, d)];
        let right = corners.iter().map(|&(a, b)| px(a, b)).fold(f64::NEG_INFINITY, f64::max);
        let left = corners.iter().map(|&(a, b)| px(a, b)).fold(f64::INFINITY, f64::min);
        let height = corners.iter().map(|&(a, b)| py(a, b)).fold(f64::NEG_INFINITY, f64::max) + oy;
        let bottom = corners.iter().map(|&(a, b)| py(a, b)).fold(f64::INFINITY, f64::min) + oy;
        let tx = -px(ox, oy) - left;
        let ty = -py(ox, oy) + oy;
        let m = self.matrix;
        TBox {
            width: right - left,
            height,
            depth: -bottom,
            matrix: [c * m[0] - s * m[1], s * m[0] + c * m[1], c * m[2] - s * m[3], s * m[2] + c * m[3], c * m[4] - s * m[5] + tx, s * m[4] + c * m[5] + ty],
        }
    }

    pub fn graphic_box(&self) -> GraphicBox {
        GraphicBox { width: self.width, height: self.height, depth: self.depth, matrix: self.matrix }
    }
}

/// `\Grot@box@kv`'s rotation point for the `Grot` keys (`origin`, `x`,
/// `y`) of a box: the centre `(\width/2, (\height-\depth)/2)` unless the
/// letters `l`/`r`/`t`/`b`/`B` or explicit lengths move it.
pub fn rotation_origin(keys: &str, width: f64, height: f64, depth: f64, env: &LengthEnv) -> (f64, f64) {
    let (mut x, mut y) = (width / 2.0, (height - depth) / 2.0);
    for entry in split_top_level(keys) {
        let (k, v) = match entry.split_once('=') {
            Some((k, v)) => (k.trim(), Some(v.trim().trim_matches(|c| c == '{' || c == '}'))),
            None => (entry.trim(), None),
        };
        match k {
            "origin" => {
                for ch in v.unwrap_or("c").chars() {
                    match ch {
                        'l' => x = 0.0,
                        'r' => x = width,
                        't' => y = height,
                        'b' => y = -depth,
                        'B' => y = 0.0,
                        _ => {}
                    }
                }
            }
            "x" => x = v.and_then(|v| parse_dimen(v, env)).unwrap_or(x),
            "y" => y = v.and_then(|v| parse_dimen(v, env)).unwrap_or(y),
            _ => {}
        }
    }
    (x, y)
}

/// One graphic after all keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedGraphic {
    pub gbox: GraphicBox,
    /// The visible part of the image's unit square `[u0, v0, u1, v1]` when
    /// pdftex.def clips it (`clip` with a viewport or trim).
    pub clip: Option<[f64; 4]>,
    pub draft: bool,
    /// The requested box before any `angle` (`\Gin@req@width` x
    /// `\Gin@req@height`, the draft frame) and its unit square's map into
    /// the final box.
    pub frame_size: (f64, f64),
    pub frame: [f64; 6],
}

/// `\includegraphics` of an image whose file size is `img_w` x `img_h` TeX
/// points, following graphicx.sty `\Gin@ii`/`\Gin@esetsize` and pdftex.def
/// `\Ginclude@@pdftex`:
///
/// * `viewport`/`trim` change the natural size to the viewport's; the image
///   is placed `\hskip-\Gin@vllx bp` and `\lower\Gin@vlly bp` in it, and
///   `clip` (or `\includegraphics*`) cuts it to the natural box;
/// * size keys before the first `angle` (or `scale`) set the requested size
///   of that natural box (`\Gin@req@sizes`: `width`/`height` win over
///   `scale`, both with `keepaspectratio` take the smaller factor);
/// * `angle` rotates the result (`\Gin@erotate`, about `origin` if given);
///   `scale` after it wraps `\Gscale@box`, and `width`/`height` left at the
///   end wrap `\Gscale@@box` of the box so far.
pub fn place_image(img_w: f64, img_h: f64, keys: &[GKey], starred: bool, draft: bool, env: &LengthEnv) -> PlacedGraphic {
    enum Op {
        Scale(f64),
        Resize(Option<f64>, Option<f64>),
        Rotate(f64),
    }
    enum Req {
        Natural,
        Scale(f64),
        Width(f64),
        Height(f64),
        Both(f64, f64),
    }
    let (mut llx, mut lly, mut urx, mut ury) = (0.0, 0.0, img_w, img_h);
    let (mut offx, mut offy) = (0.0, 0.0);
    let mut clip = starred;
    let mut draft = draft;
    let mut origin: Option<&str> = None;
    let (mut tempswa, mut iso, mut total) = (false, false, false);
    let (mut ew, mut eh): (Option<f64>, Option<f64>) = (None, None);
    let mut req = Req::Natural;
    let mut ops: Vec<Op> = Vec::new();
    let esetsize = |tempswa: bool, ew: &mut Option<f64>, eh: &mut Option<f64>, req: &mut Req, ops: &mut Vec<Op>| {
        if tempswa {
            if ew.is_some() || eh.is_some() {
                ops.push(Op::Resize(*ew, *eh));
            }
        } else {
            match (*ew, *eh) {
                (None, None) => {}
                (Some(w), None) => *req = Req::Width(w),
                (None, Some(h)) => *req = Req::Height(h),
                (Some(w), Some(h)) => *req = Req::Both(w, h),
            }
        }
        (*ew, *eh) = (None, None);
    };
    for k in keys {
        match k {
            GKey::Width(v) => ew = Some(*v),
            GKey::Height(v) => eh = Some(*v),
            GKey::TotalHeight(v) => {
                total = true;
                eh = Some(*v);
            }
            GKey::KeepAspectRatio(v) => iso = *v,
            GKey::Scale(f) => {
                if tempswa {
                    ops.push(Op::Scale(*f));
                } else {
                    req = Req::Scale(*f);
                }
                tempswa = true;
            }
            GKey::Angle(a) => {
                esetsize(tempswa, &mut ew, &mut eh, &mut req, &mut ops);
                tempswa = true;
                ops.push(Op::Rotate(*a));
            }
            GKey::Trim(t) => {
                (llx, lly, urx, ury) = (t[0], t[1], img_w - t[2], img_h - t[3]);
                (offx, offy) = (t[0], t[1]);
            }
            GKey::Viewport(v) => {
                (llx, lly, urx, ury) = (v[0], v[1], v[2], v[3]);
                (offx, offy) = (v[0], v[1]);
            }
            GKey::Clip(c) => clip = *c,
            GKey::Draft(d) => draft = *d,
            GKey::Origin(o) => origin = Some(o),
            GKey::Page(_) | GKey::Unsupported(_) => {}
        }
    }
    esetsize(tempswa, &mut ew, &mut eh, &mut req, &mut ops);
    let (nat_w, nat_h) = (urx - llx, ury - lly);
    let (sx, sy, req_w, req_h) = match req {
        Req::Natural => (1.0, 1.0, nat_w, nat_h),
        Req::Scale(f) => (f, f, f * nat_w, f * nat_h),
        Req::Width(w) => {
            let f = scale_div(w, nat_w);
            (f, f, w, f * nat_h)
        }
        Req::Height(h) => {
            let f = scale_div(h, nat_h);
            (f, f, f * nat_w, h)
        }
        Req::Both(w, h) => {
            let (mut a, mut b) = (scale_div(w, nat_w), scale_div(h, nat_h));
            if iso {
                if b > a {
                    b = a;
                } else {
                    a = b;
                }
            }
            (a, b, a * nat_w, b * nat_h)
        }
    };
    // `\Gin@setfile`: `\dp\z@\z@ \ht\z@\Gin@req@height \wd\z@\Gin@req@width`.
    let mut b = TBox { width: req_w, height: req_h, depth: 0.0, matrix: [sx * img_w, 0.0, 0.0, sy * img_h, -sx * offx, -sy * offy] };
    let mut frame = TBox { matrix: [req_w, 0.0, 0.0, req_h, 0.0, 0.0], ..b };
    let step = |b: TBox, op: &Op| match *op {
        Op::Scale(f) => b.scale(f, f),
        Op::Resize(w, h) => b.resize(w, h, total, iso),
        Op::Rotate(a) => {
            let (ox, oy) = match origin {
                Some(o) => rotation_origin(&format!("origin={o}"), b.width, b.height, b.depth, env),
                None => (0.0, 0.0),
            };
            b.rotate(a, ox, oy)
        }
    };
    for op in &ops {
        b = step(b, op);
        frame = step(frame, op);
    }
    let clip = (clip && img_w > 0.0 && img_h > 0.0).then(|| [offx / img_w, offy / img_h, (offx + nat_w) / img_w, (offy + nat_h) / img_h]);
    // A clip that keeps the whole image is no clip.
    let clip = clip.filter(|c| c[0] > 1e-9 || c[1] > 1e-9 || c[2] < 1.0 - 1e-9 || c[3] < 1.0 - 1e-9);
    PlacedGraphic { gbox: b.graphic_box(), clip, draft, frame_size: (req_w, req_h), frame: frame.matrix }
}

/// Every `\graphicspath{..}` directory list of a source (the last call
/// wins, as `\def\Ginput@path`), outside comments.
pub fn graphics_path(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = source[from..].find("\\graphicspath") {
        let abs = from + at;
        from = abs + "\\graphicspath".len();
        let line_start = source[..abs].rfind('\n').map_or(0, |n| n + 1);
        if source[line_start..abs].contains('%') {
            continue;
        }
        let rest = source[from..].trim_start();
        if !rest.starts_with('{') {
            continue;
        }
        let mut depth = 0usize;
        for (i, c) in rest.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        found = flashtex_compiler::graphics::graphics_path_entries(&rest[1..i]);
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// graphicx's extension search for a file named without one (pdfTeX
/// `\Gin@extensions` order, restricted to the formats read here).
pub const EXTENSIONS: [&str; 7] = [".pdf", ".png", ".jpg", ".jpeg", ".PDF", ".PNG", ".JPG"];

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> LengthEnv {
        LengthEnv { text_width: 469.75499, text_height: 650.43, paper_width: 614.295, paper_height: 794.96999, em: 11.74988, ex: 5.16 }
    }

    #[test]
    fn png_resolution_rounds_like_pdftex() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/floats/images/green-144dpi.png");
        let info = probe(&std::fs::read(path).unwrap(), 1).unwrap();
        assert_eq!(info.pixels, Some((300, 200)));
        assert_eq!((info.width_bp, info.height_bp), (150.0, 100.0));
    }

    #[test]
    fn pdf_cropbox_is_used() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/floats/images/box-crop.pdf");
        let info = probe(&std::fs::read(path).unwrap(), 1).unwrap();
        assert_eq!((info.width_bp, info.height_bp), (200.0, 120.0));
    }

    #[test]
    fn jpeg_density_is_read() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/floats/images/blue-96dpi.jpg");
        let info = probe(&std::fs::read(path).unwrap(), 1).unwrap();
        assert_eq!(info.pixels, Some((192, 96)));
        // sips wrote JFIF units 0 (aspect ratio only, density 96) plus an
        // EXIF XResolution of 96; pdfTeX 1.40.29 ignores both and sizes the
        // image at 72 dpi (measured: \wd = 192.71951pt = 192bp).
        assert!((info.width_bp - 192.0).abs() < 1e-9, "{info:?}");
    }

    #[test]
    fn keys_follow_graphicx_order() {
        let e = env();
        let (k, p) = parse_keys("width=4in,height=1in,keepaspectratio", &e);
        assert!(p.is_empty());
        let b = size_box(200.0 / BP_PER_PT, 120.0 / BP_PER_PT, &k);
        assert!((b.height - 72.27).abs() < 1e-6 && (b.width - 120.45).abs() < 1e-6, "{b:?}");
        // angle first, then height scales the rotated box.
        let (k, _) = parse_keys("angle=90,height=1in", &e);
        let b = size_box(144.0 / BP_PER_PT, 72.0 / BP_PER_PT, &k);
        assert!((b.height - 72.27).abs() < 1e-6 && (b.width - 36.135).abs() < 1e-6 && b.depth.abs() < 1e-9, "{b:?}");
        let (k, _) = parse_keys("width=0.5\\textwidth", &e);
        assert!((size_box(100.0, 50.0, &k).width - 234.877495).abs() < 1e-6);
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn grot_box_quarter_turn_keeps_the_reference_height() {
        // \rotatebox{90}{Up}: 20pt wide, 7pt high, 2pt deep.
        let b = TBox::content(20.0, 7.0, 2.0).rotate(90.0, 0.0, 0.0);
        assert!(close(b.width, 9.0) && close(b.height, 20.0) && close(b.depth, 0.0), "{b:?}");
        // The reference point moves right by the old height.
        assert!(close(b.matrix[4], 7.0) && close(b.matrix[5], 0.0), "{b:?}");
        // origin=c rotates about (10, 2.5): the centre stays at that height.
        let c = TBox::content(20.0, 7.0, 2.0).rotate(180.0, 10.0, 2.5);
        assert!(close(c.width, 20.0) && close(c.height, 7.0) && close(c.depth, 2.0), "{c:?}");
    }

    #[test]
    fn gscale_box_negative_factors() {
        let b = TBox::content(30.0, 7.0, 2.0).scale(-1.0, 1.0);
        assert!(close(b.width, 30.0) && close(b.matrix[0], -1.0) && close(b.matrix[4], 30.0), "{b:?}");
        let f = TBox::content(30.0, 7.0, 2.0).scale(1.0, -1.0);
        assert!(close(f.height, 2.0) && close(f.depth, 7.0), "{f:?}");
        let r = TBox::content(30.0, 7.0, 2.0).resize(None, Some(18.0), true, false);
        assert!(close(r.height + r.depth, 18.0) && close(r.width, 60.0), "{r:?}");
    }

    #[test]
    fn trim_and_clip_follow_pdftex_def() {
        let e = env();
        let (k, p) = parse_keys("trim=10 5 20 10, clip", &e);
        assert!(p.is_empty(), "{p:?}");
        let (w, h) = (80.0 / BP_PER_PT, 40.0 / BP_PER_PT);
        let g = place_image(w, h, &k, false, false, &e);
        // The box is the trimmed natural size; the image sits \hskip-10bp,
        // \lower5bp in it; the clip is that box in the unit square.
        assert!((g.gbox.width * BP_PER_PT - 50.0).abs() < 1e-9 && (g.gbox.height * BP_PER_PT - 25.0).abs() < 1e-9, "{g:?}");
        assert!((g.gbox.matrix[4] * BP_PER_PT + 10.0).abs() < 1e-9 && (g.gbox.matrix[5] * BP_PER_PT + 5.0).abs() < 1e-9, "{g:?}");
        let c = g.clip.unwrap();
        assert!(close(c[0], 0.125) && close(c[1], 0.125) && close(c[2], 0.75) && close(c[3], 0.75), "{c:?}");
        // Without clip there is nothing to cut; a full viewport is no clip.
        let (k, _) = parse_keys("trim=10 5 20 10", &e);
        assert!(place_image(w, h, &k, false, false, &e).clip.is_none());
        let (k, _) = parse_keys("viewport=0 0 80 40", &e);
        assert!(place_image(w, h, &k, true, false, &e).clip.is_none());
    }

    #[test]
    fn graphicspath_last_call_wins() {
        let src = "% \\graphicspath{{no/}}\n\\graphicspath{{a/}}\n\\graphicspath{ {b/}{c/} }\n";
        assert_eq!(graphics_path(src), vec!["b/".to_string(), "c/".to_string()]);
    }
}
