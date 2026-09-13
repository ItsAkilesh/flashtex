//! FlashTeX makeindex: an original Rust implementation of the makeindex 2.18
//! index processor (TeX Live `texk/makeindexk`), producing byte-identical
//! `.ind` and `.ilg` files, plus the LaTeX-side `.idx` writer semantics
//! ([`writer`]) and `theindex` layout parameters ([`theindex`]).
//!
//! The processor is a pure function of its inputs: no file system access,
//! no environment lookups.  Callers supply file names only for the
//! transcript text.  See `CONTRACT.md` for the adoption interface.

mod ctype;
mod engine;
mod io;
pub mod session;
mod style;
pub mod theindex;
pub mod writer;

pub use ctype::CtypeLocale;
pub use session::{IndexSession, SessionUpdate};
pub use style::Style;

use engine::{Engine, Flags, Gen, PageStart as EnginePageStart};
use io::{Transcript, cat};

/// Version banner as printed by TeX Live 2026's kpathsea build.
pub const BANNER: &str = "This is makeindex, version 2.18 [TeX Live 2026] (kpathsea + Thai support).\n";

/// A named in-memory file.  `name` is used verbatim in transcript lines.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedBytes {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl NamedBytes {
    pub fn new(name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        NamedBytes { name: name.into(), bytes: bytes.into() }
    }
}

/// `-p` handling.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PageStart {
    #[default]
    None,
    /// `-p <num>`: emitted verbatim.
    Literal(Vec<u8>),
    /// `-p any`: last page of the log plus one.
    Any,
    /// `-p odd`
    Odd,
    /// `-p even`
    Even,
}

/// Command-line switches of makeindex 2.18 that affect output.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Options {
    /// `-l`
    pub letter_ordering: bool,
    /// `-c`
    pub compress_blanks: bool,
    /// cleared by `-r`
    pub merge_page: bool,
    /// `-g`
    pub german_sort: bool,
    /// `-p`
    pub page_start: PageStart,
    /// Environment `LC_CTYPE` classification used for group headings.
    pub ctype: CtypeLocale,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            letter_ordering: false,
            compress_blanks: false,
            merge_page: true,
            german_sort: false,
            page_start: PageStart::None,
            ctype: CtypeLocale::C,
        }
    }
}

/// One makeindex invocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Job {
    /// `.idx` inputs in command-line order (names as makeindex resolves
    /// them, e.g. `doc.idx` for `makeindex doc`).
    pub inputs: Vec<NamedBytes>,
    /// Style file; `name` as kpathsea reports it (e.g. `./doc.ist`).
    pub style: Option<NamedBytes>,
    /// `<base>.log`, needed for `-p any|odd|even`.
    pub log: Option<NamedBytes>,
    pub ind_name: String,
    pub ilg_name: String,
    pub options: Options,
}

impl Job {
    /// `makeindex <base>`: reads `<base>.idx`, writes `<base>.ind`/`.ilg`.
    pub fn for_base(base: &str, idx: impl Into<Vec<u8>>) -> Self {
        Job {
            inputs: vec![NamedBytes::new(format!("{base}.idx"), idx)],
            style: None,
            log: None,
            ind_name: format!("{base}.ind"),
            ilg_name: format!("{base}.ilg"),
            options: Options::default(),
        }
    }
}

/// Result of a completed run (exit status 0).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Output {
    /// `.ind` bytes (empty when nothing was written, as in C).
    pub ind: Vec<u8>,
    /// `.ilg` bytes.
    pub ilg: Vec<u8>,
    pub accepted: i32,
    pub rejected: i32,
    /// `None` when no entries were accepted ("Nothing written").
    pub lines_written: Option<i32>,
    pub warnings: i32,
}

/// A run that makeindex aborts with exit status 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fatal {
    /// The stderr text (first line; the usage line follows in C).
    pub message: String,
    /// Transcript written before the abort.
    pub ilg: Vec<u8>,
}

impl std::fmt::Display for Fatal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.trim_end())
    }
}

impl std::error::Error for Fatal {}

pub(crate) struct Prepared {
    pub style: Style,
    pub log: Transcript,
}

/// Apply the style file (if any) and emit the banner, as `process_idx` does.
pub(crate) fn prepare(job: &Job) -> Result<(Prepared, Option<EnginePageStart>), Fatal> {
    let mut log = Transcript::new();
    let mut page = None;
    if job.inputs.is_empty() {
        return Err(Fatal { message: "stdin input is not supported by the in-process API\n".into(), ilg: Vec::new() });
    }
    match &job.options.page_start {
        PageStart::None => {}
        PageStart::Literal(p) => {
            if p.len() >= 99 {
                return Err(Fatal { message: "Page number too high\n".into(), ilg: Vec::new() });
            }
            page = Some(EnginePageStart { pageno: p.clone(), even_odd: -1 });
        }
        mode => {
            let even_odd = match mode {
                PageStart::Any => 0,
                PageStart::Odd => 1,
                _ => 2,
            };
            let base = job.inputs[0].name.rsplit_once('.').map(|(b, _)| b).unwrap_or(&job.inputs[0].name);
            let log_name = format!("{base}.log");
            let Some(l) = &job.log else {
                return Err(Fatal { message: format!("Source log file {log_name} not found.\n"), ilg: Vec::new() });
            };
            match engine::find_pageno(&l.bytes) {
                Some(p) => page = Some(EnginePageStart { pageno: p, even_odd }),
                None => {
                    log.puts(&format!("Couldn't find any page number in {log_name}...ignored\n"));
                }
            }
        }
    }
    log.puts(BANNER);
    let mut style = Style::default();
    if let Some(s) = &job.style {
        style::scan_sty(&mut style, s.name.as_bytes(), &s.bytes, &mut log);
    }
    if job.options.german_sort && style.quote == b'"' {
        return Err(Fatal {
            message: "Option -g invalid, quote character must be different from '\"'.\n".into(),
            ilg: log.buf,
        });
    }
    Ok((Prepared { style, log }, page))
}

pub(crate) fn run_prepared(job: &Job, prepared: Prepared, page: Option<EnginePageStart>) -> Output {
    let Prepared { style, log } = prepared;
    let o = &job.options;
    let flags = Flags {
        letter_ordering: o.letter_ordering,
        compress_blanks: o.compress_blanks,
        merge_page: o.merge_page,
        german_sort: o.german_sort,
        ctype: o.ctype,
    };
    let mut eng = Engine::new(&style, flags, log);
    for input in &job.inputs {
        eng.scan_idx(input.name.as_bytes(), &input.bytes);
    }
    let accepted = eng.idx_tt - eng.idx_et;
    let rejected = eng.idx_et;
    if job.inputs.len() > 1 {
        let m = format!(
            "Overall {} files read ({} entries accepted, {} rejected).\n",
            job.inputs.len(),
            accepted,
            rejected
        );
        eng.log.puts(&m);
    }
    let mut out = Output { accepted, rejected, ..Default::default() };
    if accepted > 0 && !eng.entries.is_empty() {
        let mut order: Vec<usize> = (0..eng.entries.len()).collect();
        eng.sort_idx(&mut order);
        let (ind, lines, warnings) = Gen::new(&mut eng, order, job.ind_name.as_bytes()).gen_ind(page);
        out.ind = ind;
        out.lines_written = Some(lines);
        out.warnings = warnings;
        eng.log.put(&cat(&[b"Output written in ", job.ind_name.as_bytes(), b".\n"]));
    } else {
        eng.log.put(&cat(&[b"Nothing written in ", job.ind_name.as_bytes(), b".\n"]));
    }
    eng.log.put(&cat(&[b"Transcript written in ", job.ilg_name.as_bytes(), b".\n"]));
    out.ilg = eng.log.buf;
    out
}

/// Run makeindex on in-memory inputs.
pub fn run(job: &Job) -> Result<Output, Fatal> {
    let (prepared, page) = prepare(job)?;
    Ok(run_prepared(job, prepared, page))
}

/// Parsed makeindex command line (file contents are resolved by the caller).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandLine {
    pub options: Options,
    pub style: Option<String>,
    pub ind: Option<String>,
    pub ilg: Option<String>,
    /// Input names as given (without `.idx` completion).
    pub inputs: Vec<String>,
}

impl CommandLine {
    /// Parse arguments the way `mkind.c` `main` does (lines 128-238).
    pub fn parse<I: IntoIterator<Item = S>, S: AsRef<str>>(args: I) -> Result<Self, String> {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        let mut cl = CommandLine::default();
        let mut k = 0;
        while k < args.len() {
            let a = &args[k];
            if let Some(flags) = a.strip_prefix('-') {
                if flags.is_empty() {
                    break;
                }
                for ch in flags.chars() {
                    match ch {
                        'i' => return Err("stdin input (-i) is not supported".into()),
                        'l' => cl.options.letter_ordering = true,
                        'r' => cl.options.merge_page = false,
                        'q' => {}
                        'c' => cl.options.compress_blanks = true,
                        'g' => cl.options.german_sort = true,
                        's' | 'o' | 't' | 'p' => {
                            k += 1;
                            let Some(v) = args.get(k) else {
                                return Err(match ch {
                                    's' => "Expected -s <stylefile>".into(),
                                    'o' => "Expected -o <ind>".into(),
                                    't' => "Expected -t <logfile>".into(),
                                    _ => "Expected -p <num>".into(),
                                });
                            };
                            match ch {
                                's' => cl.style = Some(v.clone()),
                                'o' => cl.ind = Some(v.clone()),
                                't' => cl.ilg = Some(v.clone()),
                                _ => {
                                    cl.options.page_start = match v.as_str() {
                                        "any" => PageStart::Any,
                                        "odd" => PageStart::Odd,
                                        "even" => PageStart::Even,
                                        _ => PageStart::Literal(v.as_bytes().to_vec()),
                                    }
                                }
                            }
                        }
                        'L' | 'T' => return Err("locale/Thai sorting (-L/-T) is not supported".into()),
                        other => return Err(format!("Unknown option -{other}.")),
                    }
                }
            } else {
                cl.inputs.push(a.clone());
            }
            k += 1;
        }
        Ok(cl)
    }

    /// Base name of the first input as `check_idx` computes it.
    pub fn base(&self) -> Option<String> {
        let f = self.inputs.first()?;
        let slash = f.rfind('/').map(|p| p + 1).unwrap_or(0);
        match f.rfind('.') {
            Some(p) if p != 0 && p >= slash => Some(f[..p].to_string()),
            _ => Some(f.clone()),
        }
    }
}
