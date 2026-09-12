//! Pins the bridge's restart/recovery contract at the library level: exactly
//! what a `Bridge` remembers across a process restart (durable store state)
//! versus what it forgets (in-memory `documents`/`anchors`), what a client
//! must resend to resume an in-flight capture, and whether the
//! `document_before_sha256` guard on a prepared edit is actually enforced or
//! merely reported. See `restart_cli.rs` for the same asymmetry reproduced
//! through genuinely separate OS processes talking the real JSON-lines wire
//! protocol, and `README.md` for the contract this pins.
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, *};
use std::{io::Cursor, path::Path};

fn capture() -> CaptureSubmit {
    let mut image = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut image, image::ImageFormat::Png)
        .unwrap();
    CaptureSubmit {
        capture_id: "restart-capture".into(),
        destination_id: "restart-anchor".into(),
        base_revision: 1,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: STANDARD.encode(image.into_inner()),
        },
        instructions: "preserve notation".into(),
    }
}
struct FakeConverter;
impl Converter for FakeConverter {
    fn convert(&self, _: &CaptureSubmit, _: &Context) -> Result<Proposal> {
        Ok(Proposal {
            latex: "$x$".into(),
            ambiguities: vec![],
            required_dependencies: vec![],
        })
    }
}
fn fresh_bridge(path: &Path) -> Bridge {
    Bridge::new(Store::open(path).unwrap())
}
fn original_document() -> Document {
    Document {
        project_id: "proj".into(),
        path: "main.tex".into(),
        revision: 1,
        text: "α target".into(),
    }
}
/// Drives one capture through submit/convert/prepare/confirm inside a single
/// `Bridge`, standing in for one process: this is the positive half of the
/// measured fact ("works completely in one process").
fn prepare_within(b: &mut Bridge) -> String {
    b.open_document(original_document()).unwrap();
    b.pin("restart-anchor", "proj", "main.tex", 1, 3, 9)
        .unwrap();
    b.receive(capture()).unwrap();
    b.convert("restart-capture", vec![], &FakeConverter)
        .unwrap();
    b.prepare_insert("restart-capture", 1, true)
        .unwrap()
        .edit_id
}

#[test]
fn full_lifecycle_succeeds_within_a_single_bridge_instance() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = fresh_bridge(dir.path());
    let edit_id = prepare_within(&mut b);
    let applied = b.confirm_insert("restart-capture", &edit_id, 2).unwrap();
    assert_eq!(applied.new_revision, 2);
    assert_eq!(b.document("proj", "main.tex").unwrap().text, "α $x$");
}

/// The negative half of the measured fact. `Bridge::documents` and
/// `Bridge::anchors` are ordinary in-memory fields (see `lib.rs`); nothing
/// about them is written to `Store`. So a freshly spawned `Bridge` against the
/// same store directory has an empty `documents` map, and `confirm_insert`'s
/// internal `self.document(...)` lookup fails closed with `document_missing`
/// instead of guessing at, or skipping, the source snapshot.
#[test]
fn capture_applied_in_a_fresh_process_without_reopening_the_document_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let edit_id = {
        let mut b = fresh_bridge(dir.path());
        prepare_within(&mut b)
        // `b` (and with it, its in-memory documents/anchors) is dropped here,
        // simulating the bridge process exiting.
    };

    let mut fresh = fresh_bridge(dir.path());
    let err = fresh
        .confirm_insert("restart-capture", &edit_id, 2)
        .unwrap_err();
    assert_eq!(err.code, "document_missing");
    assert_eq!(err.message, "Open the source snapshot on the Mac first");

    // The already-paid proposal and the prepared edit are untouched by the
    // failed attempt: nothing was lost, and nothing was corrupted. Read it
    // back through the same still-open store handle -- the store allows only
    // one owner at a time, exactly like the real Mac subprocess deployment.
    let record = fresh.store.require("restart-capture").unwrap();
    assert!(record.proposal.is_some());
    assert!(record.prepared.is_some());
    assert!(record.applied.is_none());
}

/// The documented recovery: resend `document_open` with the exact pre-edit
/// snapshot the prepared edit was computed against, then `capture_applied`
/// succeeds without a second paid conversion. Idempotent across further
/// restarts, as long as each fresh process resends that same snapshot.
#[test]
fn resending_document_open_then_capture_applied_recovers_without_reconversion() {
    let dir = tempfile::tempdir().unwrap();
    let edit_id = {
        let mut b = fresh_bridge(dir.path());
        prepare_within(&mut b)
    };

    let mut b = fresh_bridge(dir.path());
    b.open_document(original_document()).unwrap(); // recovery resend
    let applied = b.confirm_insert("restart-capture", &edit_id, 2).unwrap();
    assert_eq!(applied.new_revision, 2);
    assert_eq!(b.document("proj", "main.tex").unwrap().text, "α $x$");
    drop(b); // simulate this process exiting too, before the next one opens the store

    // A second freshly spawned process, resending the same pre-edit snapshot,
    // gets back the identical receipt rather than performing a second edit.
    let mut again = fresh_bridge(dir.path());
    again.open_document(original_document()).unwrap();
    assert_eq!(
        again
            .confirm_insert("restart-capture", &edit_id, 2)
            .unwrap(),
        applied
    );
}

/// `capture_status` only reads durable store state (`bridge.store.require`,
/// see `main.rs`'s dispatch for `capture_status`); it needs no resend at all
/// and is always safe to call first after a restart.
#[test]
fn capture_status_is_available_immediately_after_restart_without_any_resend() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = fresh_bridge(dir.path());
        prepare_within(&mut b);
    }
    let fresh = Store::open(dir.path()).unwrap();
    let record = fresh.require("restart-capture").unwrap();
    assert!(record.proposal.is_some());
    assert!(record.prepared.is_some());
    assert!(record.applied.is_none());
}

/// Replaying `capture_prepare_insert` (rather than going straight to
/// `capture_applied`) after a restart additionally requires `destination_pin`:
/// its idempotent-replay path calls `verify_proposal_context`, which calls
/// `context()`, which calls `capture_anchor()` -- and that needs the
/// in-memory anchor that only `pin()` restores. `document_open` alone is not
/// enough for this endpoint, unlike `capture_applied`.
#[test]
fn replaying_prepare_insert_after_restart_also_requires_destination_pin() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = fresh_bridge(dir.path());
        prepare_within(&mut b);
    }
    let mut b = fresh_bridge(dir.path());
    b.open_document(original_document()).unwrap();
    let err = b.prepare_insert("restart-capture", 1, true).unwrap_err();
    assert_eq!(err.code, "destination_reselection_required");

    // Resending the pin too makes the idempotent replay succeed.
    b.pin("restart-anchor", "proj", "main.tex", 1, 3, 9)
        .unwrap();
    assert!(b.prepare_insert("restart-capture", 1, true).is_ok());
}

/// The dangerous case: after a restart, the client re-supplies a document
/// under the SAME revision number, at the SAME byte length, but with
/// different bytes overlapping the prepared edit's offsets -- e.g. a stale
/// cached snapshot, or a bug that resent the wrong file. If
/// `document_before_sha256` were only *reported* on the wire and not
/// *enforced* internally, `confirm_insert` would splice the proposal into
/// this wrong text at the recorded offsets and silently corrupt it. It does
/// not: the recomputed hash is compared before any mutation happens.
#[test]
fn confirm_insert_rejects_a_same_revision_but_different_text_snapshot_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let edit_id = {
        let mut b = fresh_bridge(dir.path());
        prepare_within(&mut b)
    };

    let corrupted = Document {
        project_id: "proj".into(),
        path: "main.tex".into(),
        revision: 1,             // same revision number as the prepared edit's snapshot
        text: "α wrong!".into(), // same byte length as "α target", different bytes
    };
    assert_eq!(corrupted.text.len(), original_document().text.len());

    let mut b = fresh_bridge(dir.path());
    b.open_document(corrupted.clone()).unwrap();
    let err = b
        .confirm_insert("restart-capture", &edit_id, 2)
        .unwrap_err();
    assert_eq!(err.code, "revision_conflict");

    // No mutation occurred at the stale offsets.
    assert_eq!(b.document("proj", "main.tex").unwrap().text, corrupted.text);
    // And no insertion receipt was durably recorded.
    let record = b.store.require("restart-capture").unwrap();
    assert!(record.applied.is_none());
}
