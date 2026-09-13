//! Byte-for-byte comparison with makeindex 2.18 oracle output recorded by
//! `tools/gen_fixtures.py`.  No TeX program is run here.

use std::fs;
use std::path::{Path, PathBuf};

use flashtex_makeindex::{CommandLine, CtypeLocale, Job, NamedBytes, run};

fn fixtures() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut v: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_str().unwrap().starts_with("mi-"))
        .collect();
    v.sort();
    v
}

fn json_str(src: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":");
    let at = src.find(&pat)? + pat.len();
    let rest = src[at..].trim_start();
    if rest.starts_with("null") {
        return None;
    }
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'u' => {
                    let h: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&h, 16).ok()?)?);
                }
                o => out.push(o),
            },
            c => out.push(c),
        }
    }
    None
}

fn json_args(src: &str) -> Vec<String> {
    let at = src.find("\"args\":").unwrap();
    let open = at + src[at..].find('[').unwrap();
    let close = open + src[open..].find(']').unwrap();
    let body = &src[open + 1..close];
    let mut v = Vec::new();
    let mut rest = body;
    while let Some(q) = rest.find('"') {
        let tail = &rest[q..];
        let s = json_str(&format!("\"k\":{tail}"), "k").unwrap();
        let consumed = serde_len(tail);
        v.push(s);
        rest = &tail[consumed..];
    }
    v
}

fn serde_len(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

fn build_job(dir: &Path) -> Job {
    let case = fs::read_to_string(dir.join("case.json")).unwrap();
    let args = json_args(&case);
    let ctype = match json_str(&case, "ctype").as_deref() {
        Some("C") => CtypeLocale::C,
        _ => CtypeLocale::DarwinUtf8,
    };
    let cl = CommandLine::parse(&args).unwrap();
    let base = cl.base().unwrap();
    let inputs = cl
        .inputs
        .iter()
        .map(|name| {
            if dir.join(name).is_file() {
                NamedBytes::new(name.clone(), fs::read(dir.join(name)).unwrap())
            } else {
                let n = format!("{name}.idx");
                NamedBytes::new(n.clone(), fs::read(dir.join(&n)).unwrap())
            }
        })
        .collect();
    let style = cl.style.as_ref().map(|s| NamedBytes::new(format!("./{s}"), fs::read(dir.join(s)).unwrap()));
    let log_path = dir.join(format!("{base}.log"));
    let log = log_path.is_file().then(|| NamedBytes::new(format!("{base}.log"), fs::read(&log_path).unwrap()));
    let mut options = cl.options.clone();
    options.ctype = ctype;
    Job {
        inputs,
        style,
        log,
        ind_name: cl.ind.clone().unwrap_or(format!("{base}.ind")),
        ilg_name: cl.ilg.clone().unwrap_or(format!("{base}.ilg")),
        options,
    }
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

#[test]
fn makeindex_oracle_fixtures_byte_identical() {
    let mut failures = Vec::new();
    let all = fixtures();
    assert!(all.len() >= 60, "expected at least 60 makeindex fixtures, found {}", all.len());
    for dir in &all {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let expected = fs::read_to_string(dir.join("expected.json")).unwrap();
        let exit_one = expected.contains("\"exit\": 1");
        let job = build_job(dir);
        match run(&job) {
            Ok(out) => {
                if exit_one {
                    failures.push(format!("{name}: expected fatal exit"));
                    continue;
                }
                let ind = fs::read(dir.join("expected.ind")).unwrap_or_default();
                let ilg = fs::read(dir.join("expected.ilg")).unwrap();
                if out.ind != ind {
                    failures.push(format!("{name}: .ind differs\n--- expected\n{}\n--- actual\n{}", show(&ind), show(&out.ind)));
                }
                if out.ilg != ilg {
                    failures.push(format!("{name}: .ilg differs\n--- expected\n{}\n--- actual\n{}", show(&ilg), show(&out.ilg)));
                }
            }
            Err(f) => {
                if !exit_one {
                    failures.push(format!("{name}: unexpected fatal {}", f.message));
                    continue;
                }
                let want = json_str(&expected, "stderr").unwrap_or_default();
                if f.message.trim_end() != want {
                    failures.push(format!("{name}: fatal message {:?} != {:?}", f.message, want));
                }
            }
        }
    }
    let passed = all.len() - failures.len();
    eprintln!("makeindex oracle fixtures: {passed}/{} byte-identical", all.len());
    assert!(failures.is_empty(), "{} failing:\n{}", failures.len(), failures.join("\n\n"));
}
