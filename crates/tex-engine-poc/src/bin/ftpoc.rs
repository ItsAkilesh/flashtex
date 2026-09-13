//! `ftpoc <file.tex>`: typeset and print page/glyph JSON.
//! `ftpoc --bench <file.tex>`: cold run and a one-word-edit re-run timings.

use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (bench, path) = match args.as_slice() {
        [flag, p] if flag == "--bench" => (true, p.clone()),
        [p] => (false, p.clone()),
        _ => {
            eprintln!("usage: ftpoc [--bench] <file.tex>");
            std::process::exit(2);
        }
    };
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let t0 = Instant::now();
    let out = match flashtex_tex_engine_poc::typeset(&src) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let cold = t0.elapsed();
    if !bench {
        print!("{}", flashtex_tex_engine_poc::to_json(&out));
        return;
    }
    // One-word edit: replace the first body word of 5+ letters with "edited".
    let body = src.find("\\begin{document}").map(|i| i + 16).unwrap_or(0);
    let mut edited = src.clone();
    let bytes = src.as_bytes();
    let mut k = body;
    while k < bytes.len() {
        if bytes[k].is_ascii_lowercase() && (k == 0 || bytes[k - 1] == b' ') {
            let end = src[k..].find(|c: char| !c.is_ascii_lowercase()).map_or(src.len(), |e| k + e);
            if end - k >= 5 {
                edited.replace_range(k..end, "edited");
                break;
            }
            k = end;
        }
        k += 1;
    }
    let t1 = Instant::now();
    let out2 = flashtex_tex_engine_poc::typeset(&edited).expect("edited typeset");
    let rerun = t1.elapsed();
    let glyphs: usize = out.pages.iter().map(|p| p.glyphs.len()).sum();
    println!(
        "{path}\tpages={}\tglyphs={glyphs}\tcold_ms={:.2}\tedit_rerun_ms={:.2}\tpages_after_edit={}",
        out.pages.len(),
        cold.as_secs_f64() * 1000.0,
        rerun.as_secs_f64() * 1000.0,
        out2.pages.len()
    );
}
