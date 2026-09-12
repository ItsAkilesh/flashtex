//! Real journal recovery checks; no provider requests or filesystem fault mock.
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, *};
use std::{cell::Cell, fs, io::Cursor, path::Path};

fn capture() -> CaptureSubmit {
    let mut image = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut image, image::ImageFormat::Png)
        .unwrap();
    CaptureSubmit {
        capture_id: "recovery-capture".into(),
        destination_id: "recovery-anchor".into(),
        base_revision: 1,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: STANDARD.encode(image.into_inner()),
        },
        instructions: "preserve notation".into(),
    }
}
fn bridge(path: &Path) -> Bridge {
    let mut b = Bridge::new(Store::open(path).unwrap());
    b.open_document(Document {
        project_id: "recovery".into(),
        path: "main.tex".into(),
        revision: 1,
        text: "α target".into(),
    })
    .unwrap();
    b.pin("recovery-anchor", "recovery", "main.tex", 1, 3, 9)
        .unwrap();
    b
}
struct OfflineConverter(Cell<usize>);
impl Converter for OfflineConverter {
    fn convert(&self, _: &CaptureSubmit, _: &Context) -> Result<Proposal> {
        self.0.set(self.0.get() + 1);
        Ok(Proposal {
            latex: "$x$".into(),
            ambiguities: vec![],
            required_dependencies: vec![],
        })
    }
}
fn record_path(root: &Path) -> std::path::PathBuf {
    root.join("recovery-capture.json")
}

#[test]
fn receipt_and_proposal_survive_process_owner_reopen_without_conversion_replay() {
    let dir = tempfile::tempdir().unwrap();
    let expected;
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
        expected = b
            .convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .unwrap();
    }
    let mut b = bridge(dir.path());
    assert_eq!(b.receive(capture()).unwrap(), expected);
    let converter = OfflineConverter(Cell::new(0));
    assert_eq!(
        b.convert("recovery-capture", vec![], &converter).unwrap(),
        expected
    );
    assert_eq!(converter.0.get(), 0);
}
#[test]
fn prepared_then_applied_journal_reopens_and_replays_receipt_without_second_edit() {
    let dir = tempfile::tempdir().unwrap();
    let prepared;
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
        b.convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .unwrap();
        prepared = b.prepare_insert("recovery-capture", 1, true).unwrap();
    }
    let applied;
    {
        let mut b = bridge(dir.path());
        assert_eq!(
            b.prepare_insert("recovery-capture", 1, true).unwrap(),
            prepared
        );
        applied = b
            .confirm_insert("recovery-capture", &prepared.edit_id, 2)
            .unwrap();
        assert_eq!(b.document("recovery", "main.tex").unwrap().text, "α $x$");
    }
    let mut b = bridge(dir.path());
    // The Mac rehydrates its *already applied* source before replaying its receipt.
    b.open_document(Document {
        project_id: "recovery".into(),
        path: "main.tex".into(),
        revision: 2,
        text: "α $x$".into(),
    })
    .unwrap();
    for _ in 0..5 {
        assert_eq!(
            b.confirm_insert("recovery-capture", &prepared.edit_id, 2)
                .unwrap(),
            applied
        );
    }
    assert_eq!(b.document("recovery", "main.tex").unwrap().text, "α $x$");
    assert_eq!(
        b.confirm_insert("recovery-capture", &prepared.edit_id, 3)
            .unwrap_err()
            .code,
        "receipt_conflict"
    );
}
#[test]
fn truncated_and_non_json_records_fail_closed_without_overwriting_evidence() {
    for broken in [
        b"{\"schema_version\":1,".as_slice(),
        b"\xff\x00not-json".as_slice(),
    ] {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut b = bridge(dir.path());
            b.receive(capture()).unwrap();
        }
        fs::write(record_path(dir.path()), broken).unwrap();
        let mut b = bridge(dir.path());
        assert_eq!(b.receive(capture()).unwrap_err().code, "invalid_json");
        assert_eq!(fs::read(record_path(dir.path())).unwrap(), broken);
    }
}
#[test]
fn wrong_identity_digest_and_schema_are_rejected_after_reopen() {
    for field in ["schema_version", "request_sha256", "capture_id"] {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut b = bridge(dir.path());
            b.receive(capture()).unwrap();
        }
        let path = record_path(dir.path());
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match field {
            "schema_version" => value[field] = 2.into(),
            "capture_id" => value["capture"][field] = "other".into(),
            _ => value[field] = "not-the-digest".into(),
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let b = bridge(dir.path());
        assert_eq!(
            b.store.require("recovery-capture").unwrap_err().code,
            "invalid_journal"
        );
    }
}
#[test]
fn stale_temporary_files_never_replace_committed_record_or_acknowledge_new_capture() {
    let dir = tempfile::tempdir().unwrap();
    let expected;
    {
        let mut b = bridge(dir.path());
        expected = b.receive(capture()).unwrap();
    }
    fs::write(
        dir.path().join(".tmp-interrupted-save"),
        b"{broken replacement",
    )
    .unwrap();
    fs::write(
        dir.path().join(".tmp-uncommitted-capture"),
        serde_json::to_vec(&expected).unwrap(),
    )
    .unwrap();
    let b = bridge(dir.path());
    assert_eq!(b.store.require("recovery-capture").unwrap(), expected);
    assert!(b.store.get("uncommitted-capture").unwrap().is_none());
}
#[test]
fn exclusive_owner_releases_lock_on_drop_and_reopen_preserves_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let expected;
    {
        let mut b = bridge(dir.path());
        expected = b.receive(capture()).unwrap();
        assert_eq!(Store::open(dir.path()).err().unwrap().code, "store_in_use");
    }
    let b = bridge(dir.path());
    assert_eq!(b.store.require("recovery-capture").unwrap(), expected);
}
#[test]
fn oversized_record_is_bounded_and_rejected_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
    }
    let file = fs::File::create(record_path(dir.path())).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    let b = bridge(dir.path());
    assert_eq!(
        b.store.require("recovery-capture").unwrap_err().code,
        "invalid_journal"
    );
}
#[test]
fn structurally_valid_but_corrupt_proposal_is_rejected_on_recovery() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
        b.convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .unwrap();
    }
    let path = record_path(dir.path());
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["proposal"]["latex"] = "".into();
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    let mut b = bridge(dir.path());
    assert!(
        b.convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .is_err(),
        "Recovered journal must not return an empty proposal that Proposal::validate rejects"
    );
}

#[test]
fn contradictory_prepared_and_applied_journals_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
        b.convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .unwrap();
        let edit = b.prepare_insert("recovery-capture", 1, true).unwrap();
        b.confirm_insert("recovery-capture", &edit.edit_id, 2)
            .unwrap();
    }
    let path = record_path(dir.path());
    let good: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mutations = [
        ("/prepared/capture_id", serde_json::json!("wrong")),
        ("/prepared/edit_id", serde_json::json!("wrong")),
        ("/prepared/path", serde_json::json!("../escape.tex")),
        ("/prepared/project_id", serde_json::json!("another-project")),
        ("/prepared/start_byte", serde_json::json!(20)),
        ("/prepared/end_byte", serde_json::json!(4)),
        ("/prepared/replacement", serde_json::json!("different")),
        (
            "/prepared/document_before_sha256",
            serde_json::json!("invalid"),
        ),
        ("/applied/edit_id", serde_json::json!("wrong")),
        ("/applied/new_revision", serde_json::json!(1)),
        ("/rejected", serde_json::json!(true)),
        ("/proposal", serde_json::Value::Null),
        ("/prepared", serde_json::Value::Null),
        (
            "/destination_binding/source_sha256",
            serde_json::json!("invalid"),
        ),
        (
            "/context/dependencies/0/source_sha256",
            serde_json::json!("invalid"),
        ),
    ];
    for (pointer, replacement) in mutations {
        let mut changed = good.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(
            store.require("recovery-capture").unwrap_err().code,
            "invalid_journal",
            "{pointer}"
        );
    }
}
#[test]
fn legacy_missing_binding_is_readable_but_cannot_prepare_or_redirect() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = bridge(dir.path());
        b.receive(capture()).unwrap();
        b.convert("recovery-capture", vec![], &OfflineConverter(Cell::new(0)))
            .unwrap();
    }
    let path = record_path(dir.path());
    let mut legacy: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("destination_binding");
    legacy["context"]
        .as_object_mut()
        .unwrap()
        .remove("dependencies");
    fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let mut b = bridge(dir.path());
    assert!(b.store.require("recovery-capture").is_ok());
    assert_eq!(
        b.prepare_insert("recovery-capture", 1, true)
            .unwrap_err()
            .code,
        "destination_reselection_required"
    );
}
