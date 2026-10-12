//! The resident engine: runs of the Unicode engine in its host's own
//! process (docs/design/xetex/INCREMENTAL.md §5 step 2; DESIGN.md §5.1),
//! one after another, instead of a process per run.
//!
//! * **The format is loaded once.** A run's work before it reads the
//!   document -- `initialize` and `init_prim` over XeTeX's Unicode-sized
//!   `eqtb`, the terminal and the first line, the undump of the format --
//!   depends only on the command line, the format file and the session's
//!   clock. The first run keeps a checkpoint of the engine where the format
//!   is loaded (`changes/resident.ch`'s `flashtex_format_ready`: the word
//!   space with the scalar globals, and the state outside it that a
//!   checkpoint saves, `state.rs`), and a later run of the same command
//!   line from the same, unchanged format file restores it and runs the
//!   rest of the main program (`run_after_format`). Anything else (another
//!   job, another format file, a changed one) is a run from the start that
//!   keeps a new point.
//! * **Exits return.** `final_end` and the other exits unwind to [`Engine::run`]
//!   with [`EngineExit`] instead of ending the process (`system::exit_process`).
//! * **The terminal is captured** (no standard input: a run that asks for a
//!   line reads the end of the file, as a child whose standard input is
//!   `/dev/null` does), and so are the output's reports.
//! * **The clock is the session's** ([`Engine::clock`]): `\time`, `\day`,
//!   `\month`, `\year`, the random seed and the PDF's dates, as the pdfTeX
//!   engine's resident runs pin it (DESIGN.md §4.5), so that two runs of a
//!   session read the same clock.
//! * **A run stops when its host asks** (a newer compile, a cancel): the
//!   cancel flag is read between commands, at every macro call, at every
//!   line of input and at every page (`flashtex_poll`), and the run
//!   unwinds with [`Cancelled`].
//! * **The fonts' catalog** (the installed fonts' names, which a lookup by
//!   name reads) is kept across runs by the host ([`CatalogSource`]) while
//!   the font directories are unchanged.
//!
//! The engine's runtime (`flashtex_engine::system`: the run's
//! configuration, the resolver) is still per process, so a process runs one
//! resident engine at a time; R2 makes it per engine (PLAN.md §3.3).

use crate::fontmgr::FontCatalog;
use crate::generated::Globals;
use crate::system::{self, AlphaFile, ByteFile, FileVisit, WordFile};
use flashtex_engine::arena::CheckpointId;
use flashtex_engine::resolver::Format;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The payload a resident engine's run unwinds with when the program ends:
/// its exit status.
pub struct EngineExit(pub i32);

/// The payload a resident engine's run unwinds with when its host asked it
/// to stop.
pub struct Cancelled;

/// Where a resident engine's runs get the fonts' catalog.
pub type CatalogSource = Box<dyn FnMut() -> Arc<FontCatalog> + Send>;

/// What a resident engine's run is (`Host::resident`).
pub struct Resident {
    /// Set by the host: the run stops at its next poll.
    pub cancel: Arc<AtomicBool>,
    /// The format directory, searched first (a child's `TEXFORMATS=DIR:`).
    format_dir: Option<PathBuf>,
    /// Keep the point where the format is loaded.
    keep_point: bool,
    /// The point kept by this run, and the format file it loaded with its
    /// identity then.
    point: Option<(CheckpointId, RunState, Option<FormatFile>)>,
    catalog: Option<CatalogSource>,
}

impl Resident {
    /// The fonts' catalog for a lookup by name.
    pub(crate) fn font_catalog(&mut self) -> Arc<FontCatalog> {
        match self.catalog.as_mut() {
            Some(f) => f(),
            None => Arc::new(FontCatalog::for_lookup()),
        }
    }
}

/// The run's state outside the word space that the point keeps and a
/// checkpoint does not (`state.rs`'s `Saved` keeps the rest).
#[derive(Clone)]
struct RunState {
    start: Option<(i64, Vec<u8>)>,
    terminating: bool,
    tex_input_type: bool,
    req_engine: u8,
    bidi_warned: bool,
    saved_mapping_name: Option<Vec<u8>>,
}

impl RunState {
    fn of(g: &Globals) -> RunState {
        let h = &g.host;
        RunState {
            start: h.start.clone(),
            terminating: h.terminating,
            tex_input_type: h.tex_input_type,
            req_engine: h.req_engine,
            bidi_warned: h.bidi_warned,
            saved_mapping_name: h.saved_mapping_name.clone(),
        }
    }

    fn put(&self, g: &mut Globals) {
        let h = &mut g.host;
        h.start = self.start.clone();
        h.terminating = self.terminating;
        h.tex_input_type = self.tex_input_type;
        h.req_engine = self.req_engine;
        h.bidi_warned = self.bidi_warned;
        h.saved_mapping_name = self.saved_mapping_name.clone();
        h.first_line = None;
        h.pictures.clear();
    }
}

impl Globals {
    /// `changes/resident.ch`: the format is loaded and closed. A resident
    /// engine's run that is to keep the point takes a checkpoint here.
    pub fn flashtex_format_ready(&mut self) {
        if !self.host.resident.as_ref().is_some_and(|r| r.keep_point) {
            return;
        }
        let id = self.checkpoint();
        let st = RunState::of(self);
        // the format just loaded (`input_path` keeps the path it opened)
        let fmt = self
            .host
            .full_name_of_file
            .clone()
            .and_then(|p| Ident::of(&p).map(|i| (p, i)));
        if let Some(r) = self.host.resident.as_mut() {
            r.point = Some((id, st, fmt));
        }
    }

    /// `changes/resident.ch`: between two commands and at each macro call.
    #[inline]
    pub fn flashtex_poll(&mut self) {
        self.resident_poll();
    }

    /// A resident engine's run stops here (unwinds with [`Cancelled`]) when
    /// its host asked it to.
    #[inline]
    pub fn resident_poll(&mut self) {
        if let Some(r) = self.host.resident.as_ref() {
            if r.cancel.load(Ordering::Relaxed) {
                std::panic::resume_unwind(Box::new(Cancelled));
            }
        }
    }

    /// A format a resident engine's run finds in its format directory.
    pub(crate) fn resident_format(&self, format: Format, name: &str) -> Option<String> {
        if format != Format::Fmt || name.contains('/') {
            return None;
        }
        let dir = self.host.resident.as_ref()?.format_dir.as_ref()?;
        let p = dir.join(name);
        p.is_file().then(|| p.to_string_lossy().into_owned())
    }
}

/// The format file a run loaded, and its identity then.
type FormatFile = (String, Ident);

/// A file's identity: size, modification and status-change times, inode,
/// device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ident(u64, i128, i128, u64, u64);

impl Ident {
    fn of(path: &str) -> Option<Ident> {
        let m = std::fs::metadata(path).ok()?;
        let s = flashtex_engine::os::file_stat(&m);
        Some(Ident(s.size, s.mtime_ns, s.ctime_ns, s.ino, s.dev))
    }
}

/// What the state after the format load depends on: the command line as
/// configured (`flashtex_engine::system::run`, which includes the main
/// file's `%&` line), the first line, the directory and the format file.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PointKey {
    run: String,
    first_line: Option<Vec<u8>>,
    no_pdf: bool,
    root: PathBuf,
    format_dir: Option<PathBuf>,
}

struct Point {
    key: PointKey,
    id: CheckpointId,
    state: RunState,
    /// The format file the run loaded, and its identity then.
    format: Option<FormatFile>,
}

impl Point {
    /// The format file is the one a run would load now, unchanged.
    fn format_holds(&self) -> bool {
        let Some((path, id)) = &self.format else {
            return false;
        };
        // found in the format directory then: still there; found on the
        // default path: none in the format directory now
        if let Some(dir) = &self.key.format_dir {
            let name = Path::new(path).file_name();
            let in_dir = name.map(|n| dir.join(n));
            match &in_dir {
                Some(p) if p.to_string_lossy() == path.as_str() => {}
                Some(p) if p.is_file() => return false,
                _ => {}
            }
        }
        Ident::of(path).is_some_and(|now| now == *id)
    }
}

/// One run's request.
pub struct Spec {
    /// The command line, as xelatex would be run (`argv[0]` is `xelatex`).
    pub argv: Vec<String>,
    /// The working directory.
    pub root: PathBuf,
    /// The format directory, searched first (`None`: the default path).
    pub format_dir: Option<PathBuf>,
    /// Where the display list goes (`None`: no display list).
    pub dl: Option<Box<dyn Write + Send>>,
    /// The font formats whose programs travel in the display list (`None`:
    /// all).
    pub font_formats: Option<HashSet<String>>,
    /// The host sets it to stop the run.
    pub cancel: Arc<AtomicBool>,
    /// Keep nothing after the run (Low Memory): the engine's memory goes
    /// back, and the next run starts from the start.
    pub lean: bool,
}

/// How a run ended.
#[derive(Debug, Default)]
pub struct Report {
    /// The exit status (`None`: the run failed: see `failure`).
    pub exit_code: Option<i32>,
    /// The host asked the run to stop.
    pub cancelled: bool,
    /// The run broke off (an engine panic or the host's failure to set it
    /// up): its message.
    pub failure: Option<String>,
    /// What the run wrote to its terminal.
    pub terminal: Vec<u8>,
    /// The output's reports (`FlashTeX output: ...`).
    pub reports: Vec<String>,
    /// The run started from the kept point where the format is loaded.
    pub from_format: bool,
}

/// A resident engine: the engine kept between runs, and the point where its
/// format is loaded.
pub struct Engine {
    g: Option<Box<Globals>>,
    point: Option<Point>,
    /// The session's clock (DESIGN.md §4.5).
    pub clock: (i64, i32),
    catalog: Option<CatalogSource>,
    /// Keep and use the point after the format load (`FLASHTEX_UNICODE_FORMAT_POINT=0`
    /// turns it off: every run loads the format).
    pub keep_point: bool,
}

/// Resets every file of the engine: closed, as at the start of a run.
struct CloseFiles;

impl FileVisit for CloseFiles {
    fn alpha(&mut self, f: &mut AlphaFile) {
        *f = AlphaFile::default();
    }
    fn byte(&mut self, f: &mut ByteFile) {
        *f = ByteFile::default();
    }
    fn word(&mut self, f: &mut WordFile) {
        *f = WordFile::default();
    }
}

impl Engine {
    /// A resident engine whose session clock is `clock` (`None`: now).
    pub fn new(clock: Option<(i64, i32)>, catalog: Option<CatalogSource>) -> Engine {
        let clock = clock.unwrap_or_else(|| {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            (d.as_secs() as i64, d.subsec_micros() as i32)
        });
        Engine {
            g: None,
            point: None,
            clock,
            catalog,
            keep_point: std::env::var("FLASHTEX_UNICODE_FORMAT_POINT").map_or(true, |v| v != "0"),
        }
    }

    /// Forget the engine and its point (memory back; the next run starts
    /// from the start).
    pub fn clear(&mut self) {
        self.point = None;
        self.g = None;
    }

    /// Run the engine once, as `spec` asks.
    pub fn run(&mut self, spec: Spec) -> Report {
        if let Err(e) = std::env::set_current_dir(&spec.root) {
            return Report {
                failure: Some(format!("{}: {e}", spec.root.display())),
                ..Report::default()
            };
        }
        let c = crate::driver::command(&spec.argv);
        flashtex_engine::system::configure(c.options);
        // a directory made since the last run is searched (#1493)
        flashtex_engine::system::refresh_disk_dirs();
        let first_line = (!c.args.is_empty()).then(|| system::command_line(&c.args));
        let key = PointKey {
            run: format!("{:?}", flashtex_engine::system::run()),
            first_line: first_line.clone(),
            no_pdf: c.no_pdf,
            root: spec.root.clone(),
            format_dir: spec.format_dir.clone(),
        };
        let reuse = self.keep_point
            && self.g.is_some()
            && self
                .point
                .as_ref()
                .is_some_and(|p| p.key == key && p.format_holds());
        let resident = Box::new(Resident {
            cancel: spec.cancel.clone(),
            format_dir: spec.format_dir.clone(),
            keep_point: self.keep_point && !reuse && !spec.lean,
            point: None,
            catalog: self.catalog.take(),
        });
        let out = (!c.no_pdf)
            .then(|| Box::new(crate::out::Output::to_writer(spec.dl, spec.font_formats)));
        if !reuse {
            // the old engine's memory goes before the new one's comes
            self.clear();
            self.g = Some(Globals::new());
        }
        let g = self.g.as_mut().expect("made above");
        if reuse {
            let p = self.point.as_ref().expect("checked above");
            if let Err(e) = g.restore(p.id) {
                self.catalog = resident.catalog;
                self.clear();
                return Report {
                    failure: Some(format!("the state after the format load: {e}")),
                    ..Report::default()
                };
            }
            g.visit_files(&mut CloseFiles);
            p.state.put(g);
        }
        g.term_out = AlphaFile::captured_terminal();
        g.host.clock = Some(self.clock);
        // the point is after the run's first reading of the clock
        g.host.seed_pending = !reuse;
        g.host.resident = Some(resident);
        g.host.out = out;
        system::set_no_pdf(g, c.no_pdf);
        if !reuse {
            if let Some(l) = first_line {
                system::set_first_line(g, l);
            }
        }
        // (the result's type is spelled out: a closure ending in `!` is
        // otherwise inferred as returning `()` by the toolchain CI pins)
        let ran: std::thread::Result<std::convert::Infallible> =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if reuse {
                    g.run_after_format();
                } else {
                    g.tex_body();
                }
                system::final_end(g)
            }));
        let mut rep = Report {
            from_format: reuse,
            ..Report::default()
        };
        match ran {
            Ok(never) => match never {},
            Err(payload) => {
                if let Some(e) = payload.downcast_ref::<EngineExit>() {
                    rep.exit_code = Some(e.0);
                } else if payload.is::<Cancelled>() {
                    rep.cancelled = true;
                } else {
                    let msg = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_else(|| "the engine panicked".into());
                    rep.failure = Some(msg);
                }
            }
        }
        rep.terminal = g.term_out.take_captured();
        if let Some(o) = g.host.out.take() {
            rep.reports = o.reports.unwrap_or_default();
            // the display list's writer goes with it: its reader sees the end
        }
        // every file closed (written out), as a process's exit leaves them
        g.visit_files(&mut CloseFiles);
        let resident = g.host.resident.take();
        let mut kept = None;
        if let Some(r) = resident {
            self.catalog = r.catalog;
            kept = r.point;
        }
        if rep.failure.is_some() || spec.lean {
            // a broken run's engine is not used again; Low Memory keeps none
            self.clear();
            return rep;
        }
        if let Some((id, state, format)) = kept {
            self.point = Some(Point {
                key,
                id,
                state,
                format,
            });
        }
        rep
    }
}
