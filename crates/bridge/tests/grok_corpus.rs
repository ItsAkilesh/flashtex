//! Confirms every image in tests/grok-corpus/ actually satisfies the real
//! CaptureImage/CaptureSubmit validation the bridge applies to a live Grok
//! capture (crates/bridge/src/lib.rs). This is what makes the corpus more
//! than a folder of pictures that merely look plausible: every case is
//! proven, by the crate's own validator, to be a legal capture payload.
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{CaptureImage, CaptureSubmit};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct ManifestCase {
    id: String,
    image: String,
    mime_type: String,
}
#[derive(Deserialize)]
struct Manifest {
    cases: Vec<ManifestCase>,
}

#[test]
fn every_corpus_image_satisfies_capture_image_validate() {
    let manifest_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/grok-corpus/manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "reading {}: {e} (run tests/grok-corpus/generate.sh first)",
            manifest_path.display()
        )
    });
    let manifest: Manifest =
        serde_json::from_str(&manifest_text).expect("manifest.json must be valid JSON");
    assert!(
        manifest.cases.len() >= 20,
        "corpus looks too small ({} cases); did generate.sh run to completion?",
        manifest.cases.len()
    );

    for case in &manifest.cases {
        let image_path = manifest_path.parent().unwrap().join(&case.image);
        let bytes = fs::read(&image_path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", image_path.display()));
        let submit = CaptureSubmit {
            capture_id: "corpus-check".into(),
            destination_id: "anchor-1".into(),
            base_revision: 1,
            image: CaptureImage {
                mime_type: case.mime_type.clone(),
                data_base64: STANDARD.encode(&bytes),
            },
            instructions: String::new(),
        };
        submit.validate().unwrap_or_else(|e| {
            panic!(
                "corpus case '{}' failed CaptureImage validation: {e}",
                case.id
            )
        });
    }
}
