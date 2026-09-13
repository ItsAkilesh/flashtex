//! KC-110 proof of concept: typeset plain-text LaTeX articles by composing
//! FlashTeX's standalone, oracle-verified TeX crates. See `README.md` for
//! scope and results and `SEAMS.md` for the API mismatches found.

pub mod fonts;
pub mod linebreak;
pub mod stomach;

pub use stomach::{typeset, Glyph, Output, Page};

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// `{"page_width_sp", "page_height_sp", "pages": [{"page", "glyphs": [{font, size_sp, code, x_sp, y_sp}]}], "diagnostics"}`
pub fn to_json(out: &Output) -> String {
    let mut s = String::new();
    s.push_str(&format!("{{\"page_width_sp\":{},\"page_height_sp\":{},\"pages\":[", out.page_width, out.page_height));
    for (i, p) in out.pages.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\n{{\"page\":{},\"glyphs\":[", p.number));
        for (j, g) in p.glyphs.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "\n{{\"font\":{},\"size_sp\":{},\"code\":{},\"x_sp\":{},\"y_sp\":{}}}",
                json_str(&g.font),
                g.size,
                g.code,
                g.x,
                g.y
            ));
        }
        s.push_str("]}");
    }
    s.push_str("],\"diagnostics\":[");
    for (i, d) in out.diagnostics.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&json_str(d));
    }
    s.push_str("]}\n");
    s
}
