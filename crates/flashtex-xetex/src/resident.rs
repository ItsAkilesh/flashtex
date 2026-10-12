//! The resident engine: runs of the Unicode engine in its host's own
//! process (docs/design/xetex/INCREMENTAL.md §5 steps 2 and 3; DESIGN.md
//! §5.1), one after another, instead of a process per run.
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
//! * **S₀.** A run from the start or from the format point takes S₀ at the
//!   document's start (`crate::ckpt`, `changes/checkpoint.ch`). A later run
//!   whose S₀ key still holds -- nothing the run read before S₀ changed: an
//!   edit after the preamble -- restores S₀ and runs only the document's
//!   body (`resume_main_control`); otherwise it runs from the format point
//!   and takes a new S₀.
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
//!   name reads) is kept across runs by the host ([`Fonts`]) while the font
//!   directories are unchanged.
//!
//! The engine's runtime (`flashtex_engine::system`: the run's
//! configuration, the resolver, the read journal) is still per process (the
//! journal per thread), so a process runs one resident engine at a time, on
//! one thread; R2 makes it per engine (PLAN.md §3.3).

use crate::ckpt::{RunState, Taken, REQ_LOOKUP, S0};
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

/// Where a resident engine's runs get the installed fonts' catalog (the
/// host's, which keeps it across runs).
pub trait Fonts: Send {
    /// The catalog for a lookup by name.
    fn catalog(&mut self) -> Arc<FontCatalog>;
    /// The state of what a catalog made now is made from (the font
    /// directories): the same value, the same catalog.
    fn dirs(&mut self) -> [u64; 2];
}

/// The catalog made at every lookup, from the font directories as they are.
struct Unkept;

impl Fonts for Unkept {
    fn catalog(&mut self) -> Arc<FontCatalog> {
        Arc::new(FontCatalog::system_cached(None))
    }
    fn dirs(&mut self) -> [u64; 2] {
        // nothing watched: never the same
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        [t, !t]
    }
}

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
    /// Take S₀ (`crate::ckpt`).
    pub(crate) want_s0: bool,
    /// S₀ as this run took it, or why it could not.
    pub(crate) s0: Option<Result<Taken, String>>,
    fonts: Box<dyn Fonts>,
}

impl Resident {
    /// The fonts' catalog for a lookup by name.
    pub(crate) fn font_catalog(&mut self) -> Arc<FontCatalog> {
        self.fonts.catalog()
    }

    /// The font directories' state now (`Fonts::dirs`).
    pub(crate) fn font_dirs(&mut self) -> [u64; 2] {
        self.fonts.dirs()
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

/// Where a run started.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Start {
    /// From the start: `initialize`, the format's undump.
    #[default]
    Fresh,
    /// From the state after the format load.
    Format,
    /// From S₀: the document's body only.
    S0,
}

impl Start {
    pub fn name(self) -> &'static str {
        match self {
            Start::Fresh => "fresh",
            Start::Format => "format",
            Start::S0 => "s0",
        }
    }
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
    /// Where the run started.
    pub start: Start,
    /// Why S₀ was not used (`start` is not S₀ though one was kept), or why
    /// the run took none.
    pub s0_note: Option<String>,
    /// How long deciding where to start took (S₀'s key check).
    pub check_ms: f64,
}

/// A resident engine: the engine kept between runs, the point where its
/// format is loaded, and S₀.
pub struct Engine {
    g: Option<Box<Globals>>,
    point: Option<Point>,
    s0: Option<S0>,
    /// The session's clock (DESIGN.md §4.5).
    pub clock: (i64, i32),
    fonts: Option<Box<dyn Fonts>>,
    /// Keep and use the point after the format load (`FLASHTEX_UNICODE_FORMAT_POINT=0`
    /// turns it off: every run loads the format).
    pub keep_point: bool,
    /// Take and use S₀ (`FLASHTEX_UNICODE_S0=0` turns it off).
    pub keep_s0: bool,
    /// The output's font map index and font programs, kept across runs.
    font_cache: Arc<crate::out::fonts::FontCache>,
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

fn env_on(name: &str) -> bool {
    std::env::var(name).map_or(true, |v| v != "0")
}

impl Engine {
    /// A resident engine whose session clock is `clock` (`None`: now) and
    /// whose runs get the fonts' catalog from `fonts` (`None`: made at every
    /// lookup).
    pub fn new(clock: Option<(i64, i32)>, fonts: Option<Box<dyn Fonts>>) -> Engine {
        let clock = clock.unwrap_or_else(|| {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            (d.as_secs() as i64, d.subsec_micros() as i32)
        });
        Engine {
            g: None,
            point: None,
            s0: None,
            clock,
            fonts,
            keep_point: env_on("FLASHTEX_UNICODE_FORMAT_POINT"),
            keep_s0: env_on("FLASHTEX_UNICODE_S0"),
            font_cache: Arc::default(),
        }
    }

    /// Forget the engine, its point and S₀ (memory back; the next run
    /// starts from the start).
    pub fn clear(&mut self) {
        self.s0 = None;
        self.point = None;
        self.g = None;
    }

    /// Where this run can start, and why not from S₀ when one is kept.
    fn start_for(
        &mut self,
        key: &PointKey,
        first_line: &[u8],
        dl_on: bool,
    ) -> (Start, Option<String>) {
        let point_ok = self.keep_point
            && self.g.is_some()
            && self
                .point
                .as_ref()
                .is_some_and(|p| p.key == *key && p.format_holds());
        if !point_ok {
            self.s0 = None;
            return (Start::Fresh, None);
        }
        let Some(s0) = self.s0.as_mut() else {
            return (Start::Format, None);
        };
        let why = match s0.key.check_refresh(self.clock, first_line) {
            Err(why) => Some(why),
            Ok(()) if s0.dl_on != dl_on => {
                Some("the display list's source map is made in one run, not the other".into())
            }
            Ok(()) => match s0.font_dirs {
                None => None,
                Some(d) => match self.fonts.as_mut().map(|f| f.dirs()) {
                    Some(now) if now == d => None,
                    Some(_) => Some("the font directories changed".to_string()),
                    None => Some("the font directories are not watched".to_string()),
                },
            },
        };
        match why {
            None => (Start::S0, None),
            Some(w) => {
                self.s0 = None;
                (Start::Format, Some(w))
            }
        }
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
        let line = first_line.clone().unwrap_or_default();
        let key = PointKey {
            run: format!("{:?}", flashtex_engine::system::run()),
            first_line: first_line.clone(),
            no_pdf: c.no_pdf,
            root: spec.root.clone(),
            format_dir: spec.format_dir.clone(),
        };
        let t_check = std::time::Instant::now();
        let dl_on = spec.dl.is_some() && env_on("FLASHTEX_SOURCE_MAP");
        let (mut start, mut s0_note) = self.start_for(&key, &line, dl_on);
        let check_ms = t_check.elapsed().as_secs_f64() * 1e3;
        let mut rewrite_failed = None;
        if start == Start::S0 {
            // the files the run wrote and closed before S₀, as it left them
            if let Err(e) = self.s0.as_ref().expect("checked").key.rewrite_outputs() {
                rewrite_failed = Some(e);
            }
        }
        if let Some(e) = rewrite_failed {
            self.s0 = None;
            start = Start::Format;
            s0_note = Some(e);
        }
        // Put the kept state back: from S₀, else from the format point,
        // else from the start (a restore that fails -- a file it reopens
        // gone -- falls back to the next).
        loop {
            if start == Start::Fresh {
                // the old engine's memory goes before the new one's comes
                self.clear();
                self.g = Some(Globals::new());
                break;
            }
            let g = self.g.as_mut().expect("kept with its point");
            let restored = match start {
                Start::Format => {
                    let p = self.point.as_ref().expect("checked");
                    g.restore(p.id).map(|()| {
                        g.visit_files(&mut CloseFiles);
                        p.state.put(g);
                    })
                }
                _ => {
                    let s0 = self.s0.as_ref().expect("checked");
                    g.restore(s0.id).and_then(|()| {
                        // (the record puts the terminal back: captured, as
                        // S₀'s run had it)
                        g.term_out = AlphaFile::captured_terminal();
                        g.restore_record(&s0.rec)
                    })
                }
            };
            match restored {
                Ok(()) => break,
                Err(e) => {
                    s0_note = Some(format!("the kept state ({}): {e}", start.name()));
                    self.s0 = None;
                    start = if start == Start::S0 {
                        Start::Format
                    } else {
                        Start::Fresh
                    };
                }
            }
        }
        let want_s0 = self.keep_s0 && self.keep_point && !spec.lean && start != Start::S0;
        let resident = Box::new(Resident {
            cancel: spec.cancel.clone(),
            format_dir: spec.format_dir.clone(),
            keep_point: self.keep_point && start == Start::Fresh && !spec.lean,
            point: None,
            want_s0,
            s0: None,
            fonts: self.fonts.take().unwrap_or_else(|| Box::new(Unkept)),
        });
        let out = (!c.no_pdf).then(|| {
            Box::new(crate::out::Output::to_writer(
                spec.dl,
                spec.font_formats,
                Some(self.font_cache.clone()),
            ))
        });
        let g = self.g.as_mut().expect("made above");
        if start != Start::S0 {
            g.term_out = AlphaFile::captured_terminal();
        }
        g.host.clock = Some(self.clock);
        // the point and S₀ are after the run's first reading of the clock
        g.host.seed_pending = start == Start::Fresh;
        g.host.resident = Some(resident);
        g.host.out = out;
        // the source map, when the display list is made (a restored state
        // made it alike: `start_for`)
        let mem_len = g.mem.len();
        g.host.dl.set_on(dl_on, mem_len);
        system::set_no_pdf(g, c.no_pdf);
        // S₀ is armed by the run's first `big_switch` (`crate::ckpt`); a run
        // from S₀ asks for nothing
        g.ckpt_request = if want_s0 { REQ_LOOKUP } else { 0 };
        if start != Start::S0 {
            g.ckpt_arm_cs = 0;
            g.ckpt_arm_level = 0;
        }
        if start == Start::Fresh {
            if let Some(l) = first_line {
                system::set_first_line(g, l);
            }
        }
        if want_s0 {
            flashtex_engine::system::record_reads(true);
        }
        // the external effects (`\write18`) are the run's own: from the
        // start or the format point, none yet (a run from S₀ has S₀'s,
        // `restore_record`); else one in a body would count against every
        // later S₀ of the session
        if start != Start::S0 {
            flashtex_engine::system::truncate_external_effects(0);
        }
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match start {
                Start::Fresh => g.tex_body(),
                Start::Format => g.run_after_format(),
                Start::S0 => g.resume_main_control(),
            }
            system::final_end(g)
        }));
        if want_s0 {
            flashtex_engine::system::record_reads(false);
        }
        let mut rep = Report {
            start,
            s0_note,
            check_ms,
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
            // the font files the next run is likely to read again
            self.font_cache.retain(&o.doc.fonts.programs_read());
            rep.reports = o.reports.unwrap_or_default();
            // the display list's writer goes with it: its reader sees the end
        }
        // every file closed (written out), as a process's exit leaves them
        g.visit_files(&mut CloseFiles);
        let mut kept = None;
        let mut taken = None;
        if let Some(r) = g.host.resident.take() {
            self.fonts = Some(r.fonts);
            kept = r.point;
            taken = r.s0;
        }
        if rep.failure.is_some() || spec.lean {
            // a broken run's engine is not used again; Low Memory keeps none
            // (nor the fonts' files)
            self.clear();
            if spec.lean {
                self.font_cache = Arc::default();
            }
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
        // S₀, from a run that went on to its end (a cancelled run's journal
        // is whole up to S₀, which is all the key reads)
        match taken {
            Some(Ok(t)) => match crate::ckpt::make_key(&t, self.clock, &line) {
                Ok(k) => {
                    self.s0 = Some(S0 {
                        id: t.id,
                        rec: t.rec,
                        key: k,
                        font_dirs: t.font_dirs,
                        dl_on,
                    })
                }
                Err(e) => rep.s0_note = Some(format!("no S0: {e}")),
            },
            Some(Err(e)) => rep.s0_note = Some(format!("no S0: {e}")),
            None if want_s0 => {
                rep.s0_note = Some("no S0: the document's start was not reached".into())
            }
            None => {}
        }
        rep
    }
}
