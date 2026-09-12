//! Matching font bytes are opt-in; TFM/encoding/license already live in the peer fixture.
use flashtex_font_engine::{Face, Outlines, TrueTypeFace};
use flashtex_font_resources::{
    encoding::MappedItem,
    registry::{vf_project::*, *},
    *,
};
#[test]
#[ignore = "set FLASHTEX_LM_FONT to exact published lmroman10-regular.otf; no substitute font"]
fn matching_latin_modern_rooted_encoding_run() {
    let fixtures =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../font-engine/fixtures/tfm");
    let font_bytes =
        std::fs::read(std::env::var("FLASHTEX_LM_FONT").expect("exact matching font path"))
            .unwrap();
    assert_eq!(
        sha256(&font_bytes),
        "1aa18cfefa58132c52ce5de70db1fd1154201c19cd2b2cdaffba4906a33e6852"
    );
    let tfm = std::fs::read(fixtures.join("ec-lmr10.tfm")).unwrap();
    assert_eq!(
        sha256(&tfm),
        "cd13479f463b9a575d053dd7bf0884daa46bfdeffe4b7f537c193861652ac9e5"
    );
    let enc = std::fs::read(fixtures.join("lm-ec.enc")).unwrap();
    assert_eq!(
        sha256(&enc),
        "7f9932c402d22a937b853406cfdf4166b80260e3ff21a03fe9a4c05105a2918c"
    );
    let license = std::fs::read(fixtures.join("GUST-FONT-LICENSE.txt")).unwrap();
    assert_eq!(
        sha256(&license),
        "49ea6cb9257bbee0a3979c48a774cd221550ac1c20c95549efe45fc99cc18050"
    );
    let dir = tempfile::tempdir().unwrap();
    for (path, bytes) in [
        ("font.otf", font_bytes.as_slice()),
        ("font.tfm", tfm.as_slice()),
        ("font.enc", enc.as_slice()),
        ("LICENSE", license.as_slice()),
    ] {
        std::fs::write(dir.path().join(path), bytes).unwrap();
    }
    let face = TrueTypeFace::parse(font_bytes.clone()).unwrap();
    let metadata = LicenseMetadata {
        identifier: "LicenseRef-GUST-Font-License".into(),
        copyright: "see pinned license".into(),
        source: "published Latin Modern fixture".into(),
        text_path: "LICENSE".into(),
        text_sha256: sha256(&license),
        embedding_permission: EmbeddingPermission::Allowed,
    };
    let resource = ManifestEntry {
        path: "font.otf".into(),
        license: metadata.clone(),
        font: FontDescriptor {
            font_id: "latin-modern-roman10".into(),
            sha256: sha256(&font_bytes),
            byte_length: font_bytes.len() as u64,
            format: match face.outlines() {
                Outlines::Glyf => "static-truetype",
                Outlines::Cff => "static-cff",
            }
            .into(),
            face_index: 0,
            units_per_em: face.units_per_em().into(),
            glyph_count: face.num_glyphs().into(),
            postscript_name: face.postscript_name().into(),
        },
    };
    let binding = StyleBinding {
        family: "Explicit LM".into(),
        weight: 400,
        style: FontStyle::Upright,
    };
    std::fs::write(
        dir.path().join("fonts.json"),
        serde_json::to_vec(&RegistryManifest {
            schema_version: 1,
            entries: vec![RegistryEntry {
                binding: binding.clone(),
                resource,
            }],
        })
        .unwrap(),
    )
    .unwrap();
    let root = flashtex_project_files::ProjectRoot::open(dir.path()).unwrap();
    let registry = ProjectFontRegistry::load(&root, "fonts.json", Default::default()).unwrap();
    let tfm_asset = Asset {
        path: "font.tfm".into(),
        sha256: sha256(&tfm),
        license: metadata.clone(),
    };
    let enc_asset = Asset {
        path: "font.enc".into(),
        sha256: sha256(&enc),
        license: metadata,
    };
    let node = match face.outlines() {
        Outlines::Cff => Node::CffPhysicalEncodingAsset {
            id: "lm".into(),
            tfm: tfm_asset,
            binding,
            encoding_asset: enc_asset,
        },
        Outlines::Glyf => {
            let mapped: encoding::EncodingManifest = serde_json::from_slice(
                &std::fs::read(fixtures.join("ec-lmr10.encoding.json")).unwrap(),
            )
            .unwrap();
            Node::PhysicalEncodingAsset {
                id: "lm".into(),
                tfm: tfm_asset,
                binding,
                encoding_asset: enc_asset,
                declared_glyphs: mapped.declared_glyphs,
            }
        }
    };
    std::fs::write(
        dir.path().join("deps.json"),
        serde_json::to_vec(&DependencyManifest {
            schema_version: 2,
            registry_generation: registry.generation().into(),
            root: "lm".into(),
            nodes: vec![node],
        })
        .unwrap(),
    )
    .unwrap();
    let project =
        ResolvedVfProject::load(&root, "deps.json", &registry, Default::default()).unwrap();
    let fi = project
        .physical_run("lm", b"fi", registry.generation())
        .unwrap();
    assert!(matches!(
        fi.items.as_slice(),
        [MappedItem::Glyph {
            tfm_code: 28,
            input_start: 0,
            input_end: 2,
            ..
        }]
    ));
    let av = project
        .physical_run("lm", b"AV", registry.generation())
        .unwrap();
    assert!(matches!(
        av.items.as_slice(),
        [
            MappedItem::Glyph {
                input_start: 0,
                input_end: 1,
                ..
            },
            MappedItem::Kern(tfm::FixWord(-116509)),
            MappedItem::Glyph {
                input_start: 1,
                input_end: 2,
                ..
            }
        ]
    ));
    assert_eq!(av.project_generation, project.generation());
    assert!(project.physical_run("lm", b"fi", "stale").is_err());
}
