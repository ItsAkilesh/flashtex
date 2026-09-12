mod common;

use std::fs;

use common::{TempDir, pp};
use flashtex_project_files::{
    CurrentState, Expected, ProjectRoot, RecoveryError, RecoveryJournal, SaveError, sha256,
};

#[test]
fn journal_round_trip_list_check_restore_discard() {
    let t = TempDir::new("recovery");
    let path = pp("ch/one.tex");
    let root = ProjectRoot::open(t.root()).unwrap();
    let lock = root.lock().unwrap();
    let base = lock
        .save(&path, b"base text", Expected::NewFile, false)
        .unwrap();
    let journal = RecoveryJournal::new(&root);
    assert!(journal.list().unwrap().entries.is_empty());

    let unsaved = "base text plus edits — café 😀\n\"quoted\"\\backslash";
    let written = journal
        .record(&lock, &path, unsaved, Some(base.sha256))
        .unwrap();
    assert!(
        written
            .journal_file
            .starts_with(t.root().join(".flashtex/recovery"))
    );
    assert!(
        written
            .journal_file
            .extension()
            .is_some_and(|e| e == "json")
    );
    assert_eq!(written.text_sha256, sha256(unsaved.as_bytes()));

    let loaded = journal.load(&path).unwrap().unwrap();
    assert_eq!(loaded, written);
    let listing = journal.list().unwrap();
    assert_eq!(listing.entries.len(), 1);
    assert!(listing.malformed.is_empty());
    assert_eq!(listing.entries[0].text, unsaved);
    assert_eq!(listing.entries[0].base_sha256, Some(base.sha256));

    // Disk still equals base: safe to restore.
    let check = journal.check(&loaded).unwrap();
    assert_eq!(check.current, CurrentState::MatchesBase);
    assert!(check.safe);
    let receipt = journal
        .restore_to_disk(&lock, &loaded, false)
        .unwrap()
        .unwrap();
    assert_eq!(receipt.sha256, loaded.text_sha256);
    assert_eq!(t.read("ch/one.tex"), unsaved);
    assert!(
        journal.load(&path).unwrap().is_none(),
        "restore discards the entry"
    );
    assert!(!journal.discard(&lock, &path).unwrap());
}

#[test]
fn restore_refuses_diverged_file_unless_forced() {
    let t = TempDir::new("recovery-conflict");
    let path = pp("main.tex");
    let root = ProjectRoot::open(t.root()).unwrap();
    let lock = root.lock().unwrap();
    let base = lock.save(&path, b"base", Expected::NewFile, false).unwrap();
    let journal = RecoveryJournal::new(&root);
    let entry = journal
        .record(&lock, &path, "unsaved", Some(base.sha256))
        .unwrap();

    fs::write(t.root().join("main.tex"), "someone else").unwrap();
    let check = journal.check(&entry).unwrap();
    assert_eq!(
        check.current,
        CurrentState::Diverged(sha256(b"someone else"))
    );
    assert!(!check.safe);
    match journal.restore_to_disk(&lock, &entry, false) {
        Err(RecoveryError::Save(SaveError::Conflict(c))) => {
            assert_eq!(c.ours, Some(base.sha256));
            assert_eq!(c.theirs, Some(sha256(b"someone else")));
        }
        other => panic!("expected conflict, got {other:?}"),
    }
    assert_eq!(t.read("main.tex"), "someone else");
    assert!(
        journal.load(&path).unwrap().is_some(),
        "a refused restore keeps the entry"
    );

    journal.restore_to_disk(&lock, &entry, true).unwrap();
    assert_eq!(t.read("main.tex"), "unsaved");
    assert!(journal.load(&path).unwrap().is_none());
}

#[test]
fn restore_states_for_new_and_already_saved_files() {
    let t = TempDir::new("recovery-states");
    let root = ProjectRoot::open(t.root()).unwrap();
    let lock = root.lock().unwrap();
    let journal = RecoveryJournal::new(&root);

    // Never-saved file: safe while still missing, unsafe once something exists.
    let new_path = pp("new.tex");
    let entry = journal.record(&lock, &new_path, "fresh", None).unwrap();
    assert_eq!(
        journal.check(&entry).unwrap(),
        flashtex_project_files::RestoreCheck {
            current: CurrentState::Missing,
            safe: true
        }
    );
    t.write("new.tex", "appeared");
    assert!(!journal.check(&entry).unwrap().safe);

    // A journaled buffer that was saved after all: nothing to write.
    let saved = pp("saved.tex");
    let entry = journal
        .record(&lock, &saved, "same", Some(sha256(b"old")))
        .unwrap();
    t.write("saved.tex", "same");
    assert_eq!(
        journal.check(&entry).unwrap().current,
        CurrentState::MatchesJournal
    );
    assert!(
        journal
            .restore_to_disk(&lock, &entry, false)
            .unwrap()
            .is_none()
    );
    assert!(journal.load(&saved).unwrap().is_none());

    // Base recorded but file deleted meanwhile: not safe.
    let gone = pp("gone.tex");
    let entry = journal
        .record(&lock, &gone, "text", Some(sha256(b"base")))
        .unwrap();
    assert_eq!(
        journal.check(&entry).unwrap(),
        flashtex_project_files::RestoreCheck {
            current: CurrentState::Missing,
            safe: false
        }
    );
}

#[test]
fn malformed_and_corrupted_entries_are_reported_not_deleted() {
    let t = TempDir::new("recovery-malformed");
    let root = ProjectRoot::open(t.root()).unwrap();
    let lock = root.lock().unwrap();
    let journal = RecoveryJournal::new(&root);
    let path = pp("a.tex");
    let entry = journal.record(&lock, &path, "hello", None).unwrap();
    fs::create_dir_all(journal.dir()).unwrap();
    fs::write(journal.dir().join("junk.json"), "{not json").unwrap();
    // Truncate the real entry's text without updating the hash.
    let raw = fs::read_to_string(&entry.journal_file).unwrap();
    fs::write(&entry.journal_file, raw.replace("\"hello\"", "\"hell\"")).unwrap();

    let listing = journal.list().unwrap();
    assert!(listing.entries.is_empty());
    assert_eq!(listing.malformed.len(), 2);
    assert!(
        listing
            .malformed
            .iter()
            .any(|(_, m)| m.contains("text_sha256"))
    );
    assert!(matches!(
        journal.load(&path),
        Err(RecoveryError::Malformed { .. })
    ));
    assert!(journal.dir().join("junk.json").exists());
}
