mod common;

use std::collections::BTreeSet;
use std::fs;
use std::time::Duration;

use common::{TempDir, pp};
use flashtex_project_files::{
    ChangeKind, ConflictKind, Expected, Poller, RevisionTracker, SaveConflictKind, SaveError,
    Snapshot, save_atomic, sha256,
};

#[test]
fn atomic_save_then_external_modification_conflicts() {
    let t = TempDir::new("save");
    let path = pp("notes/main.tex");

    let r1 = save_atomic(t.root(), &path, "v1", Expected::NewFile, false).unwrap();
    assert_eq!(r1.path, path);
    assert_eq!(r1.bytes, 2);
    assert_eq!(r1.sha256, sha256(b"v1"));
    assert_eq!(t.read("notes/main.tex"), "v1");
    // No temp files left behind.
    let leftovers: Vec<_> = fs::read_dir(t.root().join("notes"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(leftovers, ["main.tex"]);

    // Saving again with the last known hash succeeds.
    let r2 = save_atomic(t.root(), &path, "v2", Expected::Hash(r1.sha256), false).unwrap();
    assert_eq!(t.read("notes/main.tex"), "v2");

    // Someone else writes the file.
    fs::write(t.root().join("notes/main.tex"), "external").unwrap();
    let err = save_atomic(t.root(), &path, "v3", Expected::Hash(r2.sha256), false).unwrap_err();
    let SaveError::Conflict(c) = err else {
        panic!("expected conflict")
    };
    assert_eq!(c.kind, SaveConflictKind::ModifiedExternally);
    assert_eq!(c.ours, Some(r2.sha256));
    assert_eq!(c.theirs, Some(sha256(b"external")));
    assert_eq!(c.size, Some(8));
    assert!(c.mtime.is_some());
    assert_eq!(
        t.read("notes/main.tex"),
        "external",
        "a refused save writes nothing"
    );

    // Force overrides.
    let r3 = save_atomic(t.root(), &path, "v3", Expected::Hash(r2.sha256), true).unwrap();
    assert_eq!(t.read("notes/main.tex"), "v3");
    assert!(r3.mtime >= r1.mtime);

    // A new-file save over an existing file is refused too.
    let err = save_atomic(t.root(), &path, "v4", Expected::NewFile, false).unwrap_err();
    assert!(
        matches!(err, SaveError::Conflict(c) if c.kind == SaveConflictKind::AlreadyExists && c.ours.is_none())
    );
    assert!(save_atomic(t.root(), &path, "v4", Expected::Any, false).is_ok());
}

#[test]
fn deleted_externally_is_reported() {
    let t = TempDir::new("deleted");
    let path = pp("a.tex");
    let r = save_atomic(t.root(), &path, "x", Expected::NewFile, false).unwrap();
    fs::remove_file(t.root().join("a.tex")).unwrap();
    let err = save_atomic(t.root(), &path, "y", Expected::Hash(r.sha256), false).unwrap_err();
    let SaveError::Conflict(c) = err else {
        panic!("expected conflict")
    };
    assert_eq!(c.kind, SaveConflictKind::DeletedExternally);
    assert_eq!(c.theirs, None);
    assert_eq!(c.size, None);
    assert!(save_atomic(t.root(), &path, "y", Expected::Hash(r.sha256), true).is_ok());
}

#[cfg(unix)]
#[test]
fn permissions_are_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let t = TempDir::new("perms");
    let path = pp("exec.tex");
    let os = t.write("exec.tex", "old");
    fs::set_permissions(&os, fs::Permissions::from_mode(0o640)).unwrap();
    save_atomic(t.root(), &path, "new", Expected::Any, false).unwrap();
    assert_eq!(
        fs::metadata(&os).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert_eq!(t.read("exec.tex"), "new");
}

#[test]
fn snapshot_diff_detects_modify_delete_create_and_ignores_same_content() {
    let t = TempDir::new("watch");
    t.write("main.tex", "m");
    t.write("a.tex", "a");
    let paths = [pp("main.tex"), pp("a.tex"), pp("later.tex")];
    let snap = Snapshot::take(t.root(), &paths).unwrap();
    assert!(snap.state(&pp("later.tex")).is_none());
    assert!(snap.diff().unwrap().is_empty());

    // Rewriting identical bytes (new mtime) is not a change.
    std::thread::sleep(Duration::from_millis(20));
    fs::write(t.root().join("a.tex"), "a").unwrap();
    assert!(snap.diff().unwrap().is_empty());

    fs::write(t.root().join("main.tex"), "changed").unwrap();
    fs::remove_file(t.root().join("a.tex")).unwrap();
    t.write("later.tex", "now");
    let diff = snap.diff().unwrap();
    let kinds: Vec<(&str, ChangeKind)> = diff
        .changes
        .iter()
        .map(|c| (c.path.as_str(), c.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            ("a.tex", ChangeKind::Deleted),
            ("later.tex", ChangeKind::Created),
            ("main.tex", ChangeKind::Modified)
        ]
    );
    let m = diff
        .changes
        .iter()
        .find(|c| c.path == pp("main.tex"))
        .unwrap();
    assert_eq!(m.before.unwrap().sha256, sha256(b"m"));
    assert_eq!(m.after.unwrap().sha256, sha256(b"changed"));

    let dirty: BTreeSet<_> = [pp("main.tex")].into_iter().collect();
    let conflicts = diff.conflicts(&dirty);
    let kinds: Vec<(&str, ConflictKind, bool)> = conflicts
        .iter()
        .map(|c| (c.path.as_str(), c.kind, c.local_dirty))
        .collect();
    assert_eq!(
        kinds,
        [
            ("a.tex", ConflictKind::DeletedExternally, false),
            ("main.tex", ConflictKind::Both, true)
        ]
    );
    let clean = diff.conflicts(&BTreeSet::new());
    assert_eq!(clean[1].kind, ConflictKind::ModifiedExternally);

    // The carried-forward snapshot is quiet.
    assert!(diff.snapshot.diff().unwrap().is_empty());
}

#[test]
fn own_writes_are_not_external_changes_and_poller_advances() {
    let t = TempDir::new("poller");
    t.write("main.tex", "m");
    let mut poller = Poller::new(Snapshot::take(t.root(), &[pp("main.tex")]).unwrap());
    let receipt = save_atomic(t.root(), &pp("main.tex"), "mine", Expected::Any, false).unwrap();
    poller.snapshot_mut().record_own_write(
        &receipt.path,
        receipt.bytes,
        receipt.mtime,
        receipt.sha256,
    );
    assert!(poller.poll().unwrap().is_empty());

    fs::write(t.root().join("main.tex"), "theirs").unwrap();
    let changes = poller.poll().unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, ChangeKind::Modified);
    assert!(
        poller.poll().unwrap().is_empty(),
        "the poller advanced its snapshot"
    );

    let mut polls = 0;
    poller.run(Duration::from_millis(1), None, |r| {
        assert!(r.unwrap().is_empty());
        polls += 1;
        polls < 3
    });
    assert_eq!(polls, 3);
}

#[test]
fn revision_tracker_follows_graph_and_saves() {
    let t = TempDir::new("rev");
    t.write("main.tex", "\\input{a}");
    t.write("a.tex", "a");
    let g = flashtex_project_files::ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    let mut tracker = RevisionTracker::new();
    assert_eq!(tracker.observe_graph(&g).len(), 2);
    assert_eq!(tracker.project_revision(), 2);
    assert_eq!(
        tracker.observe_graph(&g),
        Vec::<flashtex_project_files::ProjectPath>::new()
    );
    let r = save_atomic(
        t.root(),
        &pp("a.tex"),
        "a2",
        Expected::Hash(tracker.file(&pp("a.tex")).unwrap().sha256),
        false,
    )
    .unwrap();
    let (fr, changed) = tracker.observe_digest(&pp("a.tex"), r.sha256, r.bytes);
    assert!(changed);
    assert_eq!(fr.revision, 2);
    assert_eq!(tracker.project_revision(), 3);
}

/// Issue #45 finding 3, the revision-tracker angle: a single physical file
/// referenced under two Unicode normalizations of its name (NFC vs NFD)
/// must not inflate `RevisionTracker`'s counts. Before the fix, this
/// produced 3 tracked files (main.tex plus *two* café.tex entries) for 2
/// physical files, and `project_revision` counted the duplicate as a
/// separate content change.
#[test]
fn revision_tracker_does_not_double_count_nfc_nfd_duplicate() {
    let t = TempDir::new("rev-nfc-nfd");
    let nfc_stem = "caf\u{e9}"; // "café", 'é' precomposed (NFC)
    let nfd_stem = "cafe\u{301}"; // "café", 'e' + combining acute (NFD)
    t.write(&format!("{nfc_stem}.tex"), "Cafe content.");
    t.write(
        "main.tex",
        &format!("\\input{{{nfc_stem}}} \\input{{{nfd_stem}}}"),
    );
    let g = flashtex_project_files::ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    let mut tracker = RevisionTracker::new();
    let changed = tracker.observe_graph(&g);
    assert_eq!(
        changed.len(),
        2,
        "2 physical files (main.tex, café.tex) must be 2 tracked files, got {changed:?}"
    );
    assert_eq!(tracker.project_revision(), 2);
}
