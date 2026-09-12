use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{grok, store::Store, *};
use serde_json::json;
use std::{cell::Cell, io::Cursor};

fn capture() -> CaptureSubmit {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    CaptureSubmit {
        capture_id: "capture-1".into(),
        destination_id: "anchor-1".into(),
        base_revision: 1,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: STANDARD.encode(bytes.into_inner()),
        },
        instructions: "Keep the notation".into(),
    }
}
fn setup(path: &std::path::Path) -> Bridge {
    let mut bridge = Bridge::new(Store::open(path).unwrap());
    bridge
        .open_document(Document {
            project_id: "project".into(),
            path: "main.tex".into(),
            revision: 1,
            text: "αβ world".into(),
        })
        .unwrap();
    bridge
        .pin("anchor-1", "project", "main.tex", 1, 5, 5)
        .unwrap();
    bridge
}
struct Fake {
    calls: Cell<u32>,
    fail: bool,
    proposal: Proposal,
}
impl Converter for Fake {
    fn convert(&self, _: &CaptureSubmit, _: &Context) -> Result<Proposal> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            return Err(BridgeError::new("provider_timeout", "fixture timeout"));
        }
        Ok(self.proposal.clone())
    }
}
fn fake() -> Fake {
    Fake {
        calls: Cell::new(0),
        fail: false,
        proposal: Proposal {
            latex: "$x$".into(),
            ambiguities: vec![],
            required_dependencies: vec![],
        },
    }
}
fn fake_with(proposal: Proposal) -> Fake {
    Fake {
        calls: Cell::new(0),
        fail: false,
        proposal,
    }
}

#[test]
fn durable_receipt_and_duplicate_content_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let cap = capture();
    {
        let mut b = setup(dir.path());
        let record = b.receive(cap.clone()).unwrap();
        assert_eq!(
            record.request_sha256,
            digest(&serde_json::to_vec(&cap).unwrap())
        );
        let mut conflicting = cap.clone();
        conflicting.instructions = "different".into();
        assert_eq!(
            b.receive(conflicting).unwrap_err().code,
            "capture_id_conflict"
        );
    }
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    assert_eq!(b.receive(cap.clone()).unwrap().capture, cap);
}
#[test]
fn empty_argument_artifact_blocks_insertion_even_when_approved() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let unsafe_proposal = Proposal {
        latex: "$\\frac{\\sqrt{ }}{2}$".into(),
        ambiguities: vec!["Cannot express the Greek letter pi".into()],
        required_dependencies: vec![],
    };
    b.convert("capture-1", vec![], &fake_with(unsafe_proposal))
        .unwrap();
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "unsupported_construct_requires_confirmation"
    );
    // Still blocked, not merely durably cached from the first attempt.
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "unsupported_construct_requires_confirmation"
    );
}
#[test]
fn tagged_unsupported_ambiguity_blocks_insertion_even_with_clean_latex() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let unsafe_proposal = Proposal {
        latex: "$x$".into(),
        ambiguities: vec![
            "UNSUPPORTED: source used \\oint, which this compiler cannot render".into(),
        ],
        required_dependencies: vec![],
    };
    b.convert("capture-1", vec![], &fake_with(unsafe_proposal))
        .unwrap();
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "unsupported_construct_requires_confirmation"
    );
}
#[test]
fn ordinary_ambiguity_does_not_block_insertion() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let safe_proposal = Proposal {
        latex: "$x$".into(),
        ambiguities: vec!["AMBIGUOUS: could be a 1 or a lowercase l".into()],
        required_dependencies: vec![],
    };
    b.convert("capture-1", vec![], &fake_with(safe_proposal))
        .unwrap();
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn journal_prevents_concurrent_bridge_owners() {
    let dir = tempfile::tempdir().unwrap();
    let one = Store::open(dir.path()).unwrap();
    assert!(Store::open(dir.path()).is_err());
    drop(one);
    assert!(Store::open(dir.path()).is_ok());
}
#[test]
fn utf8_rebase_and_reviewed_idempotent_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    b.edit(&EditRequest {
        project_id: "project".into(),
        path: "main.tex".into(),
        base_revision: 1,
        revision: 2,
        start_byte: 0,
        end_byte: 0,
        replacement: "Z".into(),
    })
    .unwrap();
    let f = fake();
    let record = b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(record.context.unwrap().revision, 2);
    b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 1);
    assert_eq!(
        b.prepare_insert("capture-1", 2, false).unwrap_err().code,
        "review_required"
    );
    let edit = b.prepare_insert("capture-1", 2, true).unwrap();
    assert_eq!(edit.start_byte, 6);
    assert_eq!(b.document("project", "main.tex").unwrap().text, "Zαβ world");
    let receipt = b.confirm_insert("capture-1", &edit.edit_id, 3).unwrap();
    assert_eq!(
        b.document("project", "main.tex").unwrap().text,
        "Zαβ $x$world"
    );
    assert_eq!(
        b.confirm_insert("capture-1", &edit.edit_id, 3).unwrap(),
        receipt
    );
    assert_eq!(
        b.prepare_insert("capture-1", 3, true).unwrap_err().code,
        "already_applied"
    );
}
#[test]
fn overlapping_edit_invalidates_pinned_destination() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    b.edit(&EditRequest {
        project_id: "project".into(),
        path: "main.tex".into(),
        base_revision: 1,
        revision: 2,
        start_byte: 4,
        end_byte: 7,
        replacement: "changed".into(),
    })
    .unwrap();
    assert_eq!(
        b.convert("capture-1", vec![], &fake()).unwrap_err().code,
        "destination_reselection_required"
    );
}
#[test]
fn scalar_split_and_stale_revision_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    assert_eq!(
        b.pin("other", "project", "main.tex", 1, 1, 1)
            .unwrap_err()
            .code,
        "invalid_source_range"
    );
    assert_eq!(
        b.edit(&EditRequest {
            project_id: "project".into(),
            path: "main.tex".into(),
            base_revision: 0,
            revision: 2,
            start_byte: 0,
            end_byte: 0,
            replacement: "X".into()
        })
        .unwrap_err()
        .code,
        "revision_conflict"
    );
    let mut cap = capture();
    cap.base_revision = 0;
    assert_eq!(b.receive(cap).unwrap_err().code, "revision_conflict");
}
#[test]
fn prepared_edit_does_not_survive_unreviewed_source_change() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    b.convert("capture-1", vec![], &fake()).unwrap();
    let edit = b.prepare_insert("capture-1", 1, true).unwrap();
    b.edit(&EditRequest {
        project_id: "project".into(),
        path: "main.tex".into(),
        base_revision: 1,
        revision: 2,
        start_byte: 0,
        end_byte: 0,
        replacement: "Z".into(),
    })
    .unwrap();
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "revision_conflict"
    );
    assert_eq!(
        b.confirm_insert("capture-1", &edit.edit_id, 3)
            .unwrap_err()
            .code,
        "revision_conflict"
    );
    assert!(b.store.require("capture-1").unwrap().applied.is_none());
}
#[test]
fn invalid_images_and_paths_do_not_receive_ack() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    let mut cap = capture();
    cap.image.data_base64 = STANDARD.encode(b"not an image");
    assert_eq!(b.receive(cap).unwrap_err().code, "invalid_image");
    assert!(b.store.get("capture-1").unwrap().is_none());
    assert!(relative_path("../main.tex").is_err());
    assert!(identifier("../capture").is_err());
}
#[test]
fn provider_failure_does_not_produce_proposal_or_retry() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let f = Fake {
        calls: Cell::new(0),
        fail: true,
        proposal: fake().proposal,
    };
    assert_eq!(
        b.convert("capture-1", vec![], &f).unwrap_err().code,
        "provider_timeout"
    );
    assert_eq!(f.calls.get(), 1);
    assert!(b.store.require("capture-1").unwrap().proposal.is_none());
}
#[test]
fn context_is_bounded_on_scalar_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    let text = "α".repeat(20000);
    b.open_document(Document {
        project_id: "project".into(),
        path: "main.tex".into(),
        revision: 1,
        text,
    })
    .unwrap();
    b.pin("anchor-1", "project", "main.tex", 1, 20000, 20000)
        .unwrap();
    let context = b.context(&capture(), vec![]).unwrap();
    assert!(
        context.source_before.len() + context.selected_source.len() + context.source_after.len()
            <= MAX_CONTEXT_BYTES
    );
    assert!(context.source_before.chars().all(|c| c == 'α'));
}
#[test]
fn grok_request_and_response_are_structured_and_non_storing() {
    let dir = tempfile::tempdir().unwrap();
    let b = setup(dir.path());
    let cap = capture();
    let context = b.context(&cap, vec!["inline math".into()]).unwrap();
    let req = grok::request_body(grok::DEFAULT_MODEL, &cap, &context);
    assert_eq!(req["store"], false);
    assert_eq!(req["text"]["format"]["strict"], true);
    assert!(req["input"][1]["content"][1]["image_url"]
        .as_str()
        .unwrap()
        .starts_with("data:image/png;base64,"));
    let good = json!({"status":"completed","output":[{"type":"message","content":[{"type":"output_text","text":"{\"latex\":\"x\",\"ambiguities\":[],\"required_dependencies\":[]}"}]}]});
    assert_eq!(grok::parse_response(&good).unwrap().latex, "x");
    let bad = json!({"status":"incomplete","output":[]});
    assert_eq!(
        grok::parse_response(&bad).unwrap_err().code,
        "provider_incomplete"
    );
    let refusal = json!({"status":"completed","output":[{"type":"message","content":[{"type":"refusal","refusal":"no"}]}]});
    assert_eq!(
        grok::parse_response(&refusal).unwrap_err().code,
        "provider_refusal"
    );
}

#[test]
fn rejection_is_durable_and_prevents_conversion_and_insertion() {
    let dir = tempfile::tempdir().unwrap();
    let mut bridge = setup(dir.path());
    let capture = capture();
    bridge.receive(capture.clone()).unwrap();
    bridge.reject(&capture.capture_id).unwrap();
    bridge.reject(&capture.capture_id).unwrap();
    assert!(bridge.store.require(&capture.capture_id).unwrap().rejected);
    assert_eq!(
        bridge
            .prepare_insert(&capture.capture_id, 1, true)
            .unwrap_err()
            .code,
        "capture_rejected"
    );
    drop(bridge);
    let bridge = Bridge::new(Store::open(dir.path()).unwrap());
    assert!(bridge.store.require(&capture.capture_id).unwrap().rejected);
}

#[test]
fn restart_cannot_redirect_capture_by_reusing_anchor_id() {
    for changed in ["project", "path", "range", "source"] {
        let dir = tempfile::tempdir().unwrap();
        let mut bridge = setup(dir.path());
        bridge.receive(capture()).unwrap();
        bridge.convert("capture-1", vec![], &fake()).unwrap();
        drop(bridge);
        let mut restored = Bridge::new(Store::open(dir.path()).unwrap());
        let project = if changed == "project" {
            "other"
        } else {
            "project"
        };
        let path = if changed == "path" {
            "other.tex"
        } else {
            "main.tex"
        };
        let text = if changed == "source" {
            "αβ other"
        } else {
            "αβ world"
        };
        let start = if changed == "range" { 0 } else { 5 };
        restored
            .open_document(Document {
                project_id: project.into(),
                path: path.into(),
                revision: 1,
                text: text.into(),
            })
            .unwrap();
        restored
            .pin("anchor-1", project, path, 1, start, start)
            .unwrap();
        assert_eq!(
            restored
                .prepare_insert("capture-1", 1, true)
                .unwrap_err()
                .code,
            "destination_reselection_required",
            "{changed}"
        );
    }
}

#[test]
fn exact_anchor_rehydration_allows_review_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut bridge = setup(dir.path());
    bridge.receive(capture()).unwrap();
    bridge.convert("capture-1", vec![], &fake()).unwrap();
    drop(bridge);
    let mut restored = setup(dir.path());
    let edit = restored.prepare_insert("capture-1", 1, true).unwrap();
    assert_eq!(edit.project_id, "project");
    assert_eq!(edit.start_byte, 5);
}

fn open_preamble(b: &mut Bridge, revision: u64, value: &str) {
    b.open_document(Document {
        project_id: "project".into(),
        path: "root.tex".into(),
        revision,
        text: format!("\\newcommand{{\\notation}}{{{value}}}\\input{{main}}"),
    })
    .unwrap();
}
#[test]
fn dependency_change_requires_explicit_reconversion_and_new_review() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    open_preamble(&mut b, 1, "old");
    b.receive(capture()).unwrap();
    let f = fake();
    let first = b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(first.context.unwrap().dependencies.len(), 2);
    b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 1);
    open_preamble(&mut b, 2, "new");
    let error = b.prepare_insert("capture-1", 1, true).unwrap_err();
    assert_eq!(error.code, "proposal_context_stale");
    assert!(error.message.contains("convert"));
    assert!(b.store.require("capture-1").unwrap().prepared.is_none());
    let refreshed = b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 2);
    assert!(refreshed
        .context
        .unwrap()
        .definitions
        .iter()
        .any(|s| s.contains("{new}")));
    assert_eq!(
        b.prepare_insert("capture-1", 1, false).unwrap_err().code,
        "review_required"
    );
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn unrelated_document_does_not_invalidate_review() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let f = fake();
    b.convert("capture-1", vec![], &f).unwrap();
    b.open_document(Document {
        project_id: "project".into(),
        path: "unrelated.tex".into(),
        revision: 1,
        text: "\\def\\secret{unrelated}".into(),
    })
    .unwrap();
    b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 1);
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn restart_checks_dependency_hash_even_if_revision_is_reused() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = setup(dir.path());
        open_preamble(&mut b, 1, "old");
        b.receive(capture()).unwrap();
        b.convert("capture-1", vec![], &fake()).unwrap();
    }
    let mut b = setup(dir.path());
    open_preamble(&mut b, 1, "different content same revision");
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "proposal_context_stale"
    );
    b.convert("capture-1", vec![], &fake()).unwrap();
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn restart_missing_dependency_is_stale_and_issued_edit_is_not_reconverted() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut b = setup(dir.path());
        open_preamble(&mut b, 1, "old");
        b.receive(capture()).unwrap();
        b.convert("capture-1", vec![], &fake()).unwrap();
        b.prepare_insert("capture-1", 1, true).unwrap();
    }
    let mut b = setup(dir.path());
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "proposal_context_stale"
    );
    let f = fake();
    let saved = b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 0);
    assert!(saved.prepared.is_some());
    open_preamble(&mut b, 1, "old");
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn legacy_context_without_fingerprints_requires_reconversion() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    b.receive(capture()).unwrap();
    let mut record = b.convert("capture-1", vec![], &fake()).unwrap();
    record.context.as_mut().unwrap().dependencies.clear();
    b.store.save(&record).unwrap();
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "proposal_context_stale"
    );
    let f = fake();
    b.convert("capture-1", vec![], &f).unwrap();
    assert_eq!(f.calls.get(), 1);
    b.prepare_insert("capture-1", 1, true).unwrap();
}
#[test]
fn failed_refresh_keeps_stale_proposal_unpreparable() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = setup(dir.path());
    open_preamble(&mut b, 1, "old");
    b.receive(capture()).unwrap();
    let old = b.convert("capture-1", vec![], &fake()).unwrap();
    open_preamble(&mut b, 2, "new");
    let failing = Fake {
        calls: Cell::new(0),
        fail: true,
        proposal: fake().proposal,
    };
    assert_eq!(
        b.convert("capture-1", vec![], &failing).unwrap_err().code,
        "provider_timeout"
    );
    assert_eq!(b.store.require("capture-1").unwrap().context, old.context);
    assert_eq!(
        b.prepare_insert("capture-1", 1, true).unwrap_err().code,
        "proposal_context_stale"
    );
}
