//! `flashtex-project-files --root DIR`: private JSON Lines host for the rooted
//! read/status/save primitives of `flashtex_project_files` (README, "Native
//! project integration proposal", step 2). One process per project root; the
//! Mac shell drives it over private pipes exactly like `flashtex-edit-ledger`.
//!
//! Protocol (`project-files-v1`), one JSON object per line, replies in request
//! order, every reply echoing `id`:
//!
//! - `{"id","operation":"ping"}` → `{"id","payload":{"protocol","root","pid"}}`
//! - `{"id","operation":"read","path"}` → payload `{"path","exists":bool,
//!   "text"?,"sha256"?,"bytes"?,"mtime_unix_ms"?}`
//! - `{"id","operation":"status","path","expected_sha256"?:hex|null}` →
//!   payload `{"path","exists","state":"unchanged"|"modified"|"deleted"|"created",
//!   "sha256"?,"bytes"?,"mtime_unix_ms"?}` (`state` is relative to
//!   `expected_sha256`; `null`/absent means the caller expects no file)
//! - `{"id","operation":"save","path","text","expected":"new"|"any"|hex,
//!   "force"?:bool}` → payload `{"outcome":"saved","receipt":{"path","bytes",
//!   "sha256","mtime_unix_ms"}}` or `{"outcome":"conflict","conflict":{"path",
//!   "kind":"modified_externally"|"deleted_externally"|"already_exists"|
//!   "modified_during_save","ours"?,"theirs"?,"mtime_unix_ms"?,"size"?}}`
//! - `{"id","operation":"remove","path"}` → payload `{"path","removed":bool}`
//!   (`removed:false` when nothing was there — an absent file is not an error)
//! - `{"id","operation":"rename","from","to"}` → payload
//!   `{"outcome":"renamed","from","to"}` or `{"outcome":"conflict","conflict":
//!   {"path","kind":"deleted_externally"|"already_exists","ours":null,
//!   "theirs":null,"mtime_unix_ms":null,"size":null}}`. One no-replace rename
//!   inside one directory: `from` and `to` must share a parent directory, the
//!   new name is never clobbered, and the two names never both exist. A
//!   missing `from` is `deleted_externally` and an occupied `to` is
//!   `already_exists` — conflicts, not errors, with no hashes or sizes because
//!   neither name is opened or read (`ProjectLock::rename`).
//! - `{"id","operation":"list","path"?}` → payload `{"path","files":[…],
//!   "truncated":bool}`. `path` (project-relative directory; omitted or null
//!   for the whole root) is echoed back. `files` is every project file
//!   (`PROJECT_FILE_EXTENSIONS` below) under it, found by a rooted,
//!   symlink-refusing recursive walk (`ProjectRoot::list_files`) that never
//!   leaves `--root`, as root-relative `ProjectPath` strings sorted in
//!   `ProjectPath` order. A symlink anywhere in the tree, a hidden entry
//!   (name starting with `.` — this is what excludes `.flashtex/` itself, not
//!   a special case for that name), and anything neither a regular file nor a
//!   directory are silently excluded rather than refused: a listing
//!   enumerates entries nobody named, so one such entry does not fail the
//!   whole request the way it would for `read`/`save`/`remove`/`rename`'s
//!   single caller-named path. Capped at `LIST_LIMIT` files; `truncated`
//!   says whether the cap was hit before the whole tree was walked.
//!
//! Errors are `{"id","error":{"code","message"}}` with codes `invalid_request`,
//! `invalid_path`, `refused` (symlink component, escape, not a regular file,
//! too large, lock held, unsupported target), `invalid_utf8`, `io`,
//! `directory_sync` (rename landed, durability unknown) and `line_too_long`.
//! A conflict is never an error: the shell must show it and keep its buffer.
//! Nothing here follows a symlink or writes outside `--root`; the guarantees
//! are the library's (README "Guarantees" / "Non-guarantees").

use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use flashtex_project_files::json::Json;
use flashtex_project_files::{
    DEFAULT_READ_LIMIT, Expected, ProjectPath, ProjectRoot, SaveConflict, SaveConflictKind,
    SaveError, sha256_from_hex, sha256_to_hex,
};

/// Same bound as the Mac `LineProcessClient` / transfer-v1 (12 MiB).
const MAX_LINE_BYTES: usize = 12 * 1024 * 1024;
const PROTOCOL: &str = "project-files-v1";

/// Extensions `list` reports — the LaTeX source types a project file browser
/// can meaningfully open and edit, not media pulled in via
/// `\includegraphics` (those are hashed for identity by `ProjectGraph`, never
/// opened as a document).
const PROJECT_FILE_EXTENSIONS: &[&str] = &["tex", "bib", "sty", "cls", "bst", "clo"];

/// Cap on the number of files one `list` reply reports. Matches
/// [`flashtex_project_files::DEFAULT_LIST_LIMIT`].
const LIST_LIMIT: usize = flashtex_project_files::DEFAULT_LIST_LIMIT;

struct Failure {
    code: &'static str,
    message: String,
}

fn fail(code: &'static str, message: impl Into<String>) -> Failure {
    Failure {
        code,
        message: message.into(),
    }
}

impl From<SaveError> for Failure {
    fn from(e: SaveError) -> Self {
        match e {
            SaveError::Refused(r) => fail("refused", format!("{r:?}")),
            SaveError::Io(io) if io.kind() == io::ErrorKind::InvalidData => {
                fail("invalid_utf8", io.to_string())
            }
            SaveError::Io(io) => fail("io", io.to_string()),
            SaveError::DirectorySync(io) => fail("directory_sync", io.to_string()),
            // Conflicts are payloads; `save` handles them before this conversion.
            SaveError::Conflict(c) => fail("refused", format!("{:?}", c.kind)),
        }
    }
}

fn unix_ms(t: SystemTime) -> Json {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => Json::from(d.as_millis().min(u64::MAX as u128) as u64),
        Err(_) => Json::from(0u64),
    }
}

fn field<'a>(req: &'a Json, key: &str) -> Result<&'a Json, Failure> {
    req.get(key)
        .filter(|v| !matches!(v, Json::Null))
        .ok_or_else(|| fail("invalid_request", format!("missing field {key:?}")))
}

fn string_field<'a>(req: &'a Json, key: &str) -> Result<&'a str, Failure> {
    field(req, key)?
        .as_str()
        .ok_or_else(|| fail("invalid_request", format!("field {key:?} must be a string")))
}

fn project_path_field(req: &Json, key: &str) -> Result<ProjectPath, Failure> {
    let raw = string_field(req, key)?;
    ProjectPath::normalize(raw).map_err(|e| fail("invalid_path", format!("{raw:?}: {e}")))
}

fn project_path(req: &Json) -> Result<ProjectPath, Failure> {
    project_path_field(req, "path")
}

/// The `conflict` object shared by `save` and `rename`. A conflict is never an
/// error: nothing was written or renamed and the shell must show it and keep
/// its buffer. `rename` leaves `ours`/`theirs`/`mtime_unix_ms`/`size` null —
/// it classifies entries with `fstatat` and never opens or reads them.
fn conflict_json(c: &SaveConflict) -> Json {
    let mut j = Json::object();
    j.insert("path", c.path.as_str())
        .insert(
            "kind",
            match c.kind {
                SaveConflictKind::ModifiedExternally => "modified_externally",
                SaveConflictKind::DeletedExternally => "deleted_externally",
                SaveConflictKind::AlreadyExists => "already_exists",
                SaveConflictKind::ModifiedDuringSave => "modified_during_save",
            },
        )
        .insert("ours", c.ours.as_ref().map(sha256_to_hex))
        .insert("theirs", c.theirs.as_ref().map(sha256_to_hex))
        .insert("mtime_unix_ms", c.mtime.map(unix_ms))
        .insert("size", c.size);
    j
}

fn read(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let path = project_path(req)?;
    let mut payload = Json::object();
    payload.insert("path", path.as_str());
    match root.read_text(&path, DEFAULT_READ_LIMIT)? {
        None => {
            payload.insert("exists", false);
        }
        Some((text, read)) => {
            payload
                .insert("exists", true)
                .insert("bytes", read.bytes.len() as u64)
                .insert("sha256", sha256_to_hex(&read.sha256))
                .insert("mtime_unix_ms", unix_ms(read.mtime))
                .insert("text", text);
        }
    }
    Ok(payload)
}

fn status(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let path = project_path(req)?;
    let expected = match req.get("expected_sha256") {
        None | Some(Json::Null) => None,
        Some(v) => {
            let hex = v.as_str().ok_or_else(|| {
                fail(
                    "invalid_request",
                    "expected_sha256 must be a hex string or null",
                )
            })?;
            Some(sha256_from_hex(hex).ok_or_else(|| {
                fail(
                    "invalid_request",
                    format!("expected_sha256 {hex:?} is not a SHA-256 hex digest"),
                )
            })?)
        }
    };
    let mut payload = Json::object();
    payload.insert("path", path.as_str());
    match root.read(&path, DEFAULT_READ_LIMIT)? {
        None => {
            payload.insert("exists", false).insert(
                "state",
                if expected.is_some() {
                    "deleted"
                } else {
                    "unchanged"
                },
            );
        }
        Some(read) => {
            let state = match expected {
                None => "created",
                Some(e) if e == read.sha256 => "unchanged",
                Some(_) => "modified",
            };
            payload
                .insert("exists", true)
                .insert("state", state)
                .insert("bytes", read.bytes.len() as u64)
                .insert("sha256", sha256_to_hex(&read.sha256))
                .insert("mtime_unix_ms", unix_ms(read.mtime));
        }
    }
    Ok(payload)
}

fn save(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let path = project_path(req)?;
    let text = string_field(req, "text")?;
    let expected = match string_field(req, "expected")? {
        "new" => Expected::NewFile,
        "any" => Expected::Any,
        hex => Expected::Hash(sha256_from_hex(hex).ok_or_else(|| {
            fail(
                "invalid_request",
                format!("expected {hex:?} is not \"new\", \"any\" or a SHA-256 hex digest"),
            )
        })?),
    };
    let force = match req.get("force") {
        None | Some(Json::Null) => false,
        Some(Json::Bool(b)) => *b,
        Some(_) => return Err(fail("invalid_request", "force must be a boolean")),
    };
    let mut payload = Json::object();
    match root.save(&path, text.as_bytes(), expected, force) {
        Ok(receipt) => {
            let mut r = Json::object();
            r.insert("path", receipt.path.as_str())
                .insert("bytes", receipt.bytes)
                .insert("sha256", receipt.sha256_hex())
                .insert("mtime_unix_ms", unix_ms(receipt.mtime));
            payload.insert("outcome", "saved").insert("receipt", r);
        }
        Err(SaveError::Conflict(c)) => {
            payload
                .insert("outcome", "conflict")
                .insert("conflict", conflict_json(&c));
        }
        Err(e) => return Err(e.into()),
    }
    Ok(payload)
}

/// `remove`: unlink one rooted, project-relative regular file. An absent file
/// is `removed:false`, not an error — the library's own `Ok(false)` case, so a
/// shell that deletes a file someone else already deleted converges instead of
/// failing. A symlink or non-regular entry is `refused` and left in place.
fn remove(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let path = project_path(req)?;
    let removed = root.remove(&path)?;
    let mut payload = Json::object();
    payload.insert("path", path.as_str()).insert("removed", removed);
    Ok(payload)
}

/// `rename`: one no-replace rename of a rooted, project-relative regular file
/// within its own directory. See `ProjectLock::rename` for the exact
/// guarantee: `to` is never clobbered, the two names never both exist, and the
/// residual is the source-side check-then-rename window every other operation
/// in this crate already states.
///
/// A cross-directory `from`/`to` pair is rejected here, before the library is
/// called, so the shell gets `invalid_request` (a malformed request) rather
/// than the library's `InvalidInput` I/O error reported as `io`.
fn rename(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let from = project_path_field(req, "from")?;
    let to = project_path_field(req, "to")?;
    if from.parent_dir() != to.parent_dir() {
        return Err(fail(
            "invalid_request",
            format!(
                "rename is an in-place rename within one project directory: {from:?} and {to:?} \
                 are in different directories"
            ),
        ));
    }
    let mut payload = Json::object();
    match root.rename(&from, &to) {
        Ok(()) => {
            payload
                .insert("outcome", "renamed")
                .insert("from", from.as_str())
                .insert("to", to.as_str());
        }
        Err(SaveError::Conflict(c)) => {
            payload
                .insert("outcome", "conflict")
                .insert("conflict", conflict_json(&c));
        }
        Err(e) => return Err(e.into()),
    }
    Ok(payload)
}

/// `list`: every project file under `path` (the whole root when omitted),
/// found by `ProjectRoot::list_files`. See the module doc comment for exactly
/// what is excluded and why.
fn list(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    let subdir = match req.get("path") {
        None | Some(Json::Null) => None,
        Some(_) => Some(project_path_field(req, "path")?),
    };
    let listing = root.list_files(subdir.as_ref(), PROJECT_FILE_EXTENSIONS, LIST_LIMIT)?;
    let files: Vec<Json> = listing
        .files
        .iter()
        .map(|p| Json::from(p.as_str()))
        .collect();
    let mut payload = Json::object();
    payload
        .insert(
            "path",
            subdir.as_ref().map(|p| p.as_str().to_string()).unwrap_or_default(),
        )
        .insert("files", files)
        .insert("truncated", listing.truncated);
    Ok(payload)
}

fn handle(root: &ProjectRoot, req: &Json) -> Result<Json, Failure> {
    match string_field(req, "operation")? {
        "ping" => {
            let mut p = Json::object();
            p.insert("protocol", PROTOCOL)
                .insert("root", root.path().to_string_lossy().into_owned())
                .insert("pid", u64::from(std::process::id()));
            Ok(p)
        }
        "read" => read(root, req),
        "status" => status(root, req),
        "save" => save(root, req),
        "remove" => remove(root, req),
        "rename" => rename(root, req),
        "list" => list(root, req),
        other => Err(fail(
            "unsupported_operation",
            format!("unknown operation {other:?}"),
        )),
    }
}

fn reply(out: &mut impl Write, id: Json, result: Result<Json, Failure>) -> io::Result<()> {
    let mut line = Json::object();
    line.insert("id", id);
    match result {
        Ok(payload) => {
            line.insert("payload", payload);
        }
        Err(f) => {
            let mut e = Json::object();
            e.insert("code", f.code).insert("message", f.message);
            line.insert("error", e);
        }
    }
    writeln!(out, "{}", line.to_string_compact())?;
    out.flush()
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let root_arg = match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(dir), None) if flag == "--root" => PathBuf::from(dir),
        _ => {
            eprintln!("usage: flashtex-project-files --root PROJECT_DIRECTORY");
            std::process::exit(2);
        }
    };
    let root = match ProjectRoot::open(&root_arg) {
        Ok(root) => root,
        Err(e) => {
            eprintln!(
                "flashtex-project-files: cannot open root {}: {e}",
                root_arg.display()
            );
            std::process::exit(1);
        }
    };
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let n = match input
            .by_ref()
            .take(MAX_LINE_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)
        {
            Ok(n) => n,
            Err(e) => {
                eprintln!("flashtex-project-files: stdin: {e}");
                std::process::exit(1);
            }
        };
        if n == 0 {
            return;
        }
        if line.len() > MAX_LINE_BYTES {
            let _ = reply(
                &mut output,
                Json::Null,
                Err(fail(
                    "line_too_long",
                    format!("request exceeds {MAX_LINE_BYTES} bytes"),
                )),
            );
            std::process::exit(1);
        }
        let text = match std::str::from_utf8(&line) {
            Ok(t) => t.trim_end_matches(['\n', '\r']),
            Err(_) => {
                let _ = reply(
                    &mut output,
                    Json::Null,
                    Err(fail("invalid_request", "request is not UTF-8")),
                );
                continue;
            }
        };
        if text.trim().is_empty() {
            continue;
        }
        let req = match Json::parse(text) {
            Ok(j) => j,
            Err(e) => {
                let _ = reply(
                    &mut output,
                    Json::Null,
                    Err(fail("invalid_request", format!("malformed JSON: {e}"))),
                );
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Json::Null);
        let result = handle(&root, &req);
        if let Err(e) = reply(&mut output, id, result) {
            // The reader went away; there is nobody to tell.
            eprintln!("flashtex-project-files: stdout: {e}");
            std::process::exit(1);
        }
    }
}
