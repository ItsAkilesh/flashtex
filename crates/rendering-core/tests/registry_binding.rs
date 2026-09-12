use flashtex_font_engine::{Face, ShapeOptions, TrueTypeFace};
use flashtex_font_resources::{cff::HintPolicy, engine_adapter::*, registry::*, *};
use flashtex_project_files::ProjectRoot;
use flashtex_rendering_core::{
    batch::ExactClip,
    digest,
    outlines::{OutlineCoordinate, OutlinePoint},
    registry_binding::*,
    shaped_run::*,
    SourceSnapshot,
};
use std::sync::Arc;
#[allow(dead_code)]
#[path = "support/font_fixture.rs"]
mod font_fixture;
fn selection(family: &str) -> StyleBinding {
    StyleBinding {
        family: family.into(),
        weight: 400,
        style: FontStyle::Upright,
    }
}
fn entry(bytes: &[u8], name: &str, format: &str, license: &[u8]) -> RegistryEntry {
    let face = TrueTypeFace::parse(bytes.to_vec()).unwrap();
    RegistryEntry {
        binding: selection(name),
        resource: ManifestEntry {
            font: FontDescriptor {
                font_id: name.into(),
                sha256: digest(bytes),
                byte_length: bytes.len() as u64,
                format: format.into(),
                face_index: 0,
                units_per_em: face.units_per_em() as u32,
                glyph_count: face.num_glyphs() as u32,
                postscript_name: face.postscript_name().into(),
            },
            path: format!("{name}.font"),
            license: LicenseMetadata {
                identifier: "LicenseRef-Fixture".into(),
                copyright: "See pinned fixture license".into(),
                source: "explicit fixture resource".into(),
                text_path: format!("{name}.license"),
                text_sha256: digest(license),
                embedding_permission: EmbeddingPermission::Unknown,
            },
        },
    }
}
fn save(dir: &std::path::Path, m: &RegistryManifest) {
    std::fs::write(dir.join("fonts.json"), serde_json::to_vec(m).unwrap()).unwrap();
}
fn load(root: &ProjectRoot) -> Arc<ProjectFontRegistry> {
    Arc::new(ProjectFontRegistry::load(root, "fonts.json", RegistryLimits::default()).unwrap())
}
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn placement() -> Placement {
    Placement {
        item_index: 1,
        size: r(1000, 3),
        origin: OutlinePoint {
            x: r(1, 2),
            y: r(7, 4),
        },
        clip: ExactClip {
            left: r(-100000, 1),
            top: r(-100000, 1),
            right: r(100000, 1),
            bottom: r(100000, 1),
        },
    }
}
fn limits() -> PlacementLimits {
    PlacementLimits {
        max_glyphs: 100,
        max_commands: 10000,
        max_payload_bytes: 1024 * 1024,
    }
}
fn frame(renderer: &RegistryRenderer, lease: &RenderLease) -> RegistryFrame {
    let s = SourceSnapshot {
        revision: 1,
        text: "AA".into(),
    };
    let run = renderer
        .shape(
            lease,
            ShapeRequest {
                source: &s.text,
                source_sha256: &digest(s.text.as_bytes()),
                path: "main.tex",
                revision: 1,
                range: 0..2,
                font_sha256: &lease.binding().declaration.font.sha256,
                face_index: 0,
                encoding: InputEncoding::Unicode,
                variation_coordinates: &[],
                options: ShapeOptions::PLAIN,
            },
        )
        .unwrap();
    renderer
        .place(lease, &run, &s, placement(), limits(), HintPolicy::Unhinted)
        .unwrap()
}
#[test]
fn resource_replacement_invalidates_leases_and_cache_but_retains_frames() {
    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let font = font_fixture::shaping_fixture();
    let mut manifest = RegistryManifest {
        schema_version: 1,
        entries: vec![
            entry(&font, "body", "static-truetype", b"test"),
            entry(&font, "heading", "static-truetype", b"test"),
        ],
    };
    for e in &manifest.entries {
        std::fs::write(dir.path().join(&e.resource.path), &font).unwrap();
        std::fs::write(dir.path().join(&e.resource.license.text_path), b"test").unwrap();
    }
    save(dir.path(), &manifest);
    let first = load(&root);
    let mut renderer = RegistryRenderer::new(
        "project-instance",
        first.clone(),
        RegistryRenderLimits {
            max_bindings: 2,
            max_cache_bytes: 100000,
        },
    )
    .unwrap();
    let body = renderer
        .bind(&selection("body"), first.generation())
        .unwrap();
    let heading = renderer
        .bind(&selection("heading"), first.generation())
        .unwrap();
    let held = frame(&renderer, &body);
    let other = frame(&renderer, &heading);
    assert_ne!(held.binding().selection, other.binding().selection);
    assert_eq!(held.run().advance(), other.run().advance());
    let bytes = held.replay_bytes(100000).unwrap();
    let snapshot = SourceSnapshot {
        revision: 1,
        text: "AA".into(),
    };
    renderer
        .verify_replay(&body, &bytes, "main.tex", &snapshot)
        .unwrap();
    assert!(renderer
        .verify_replay(&heading, &bytes, "main.tex", &snapshot)
        .is_err());
    assert_eq!(renderer.cached_bindings(), 2);
    assert!(!renderer.replace(load(&root)).unwrap());
    // Change actual hmtx bytes and the explicit resource declaration.
    let mut replacement = font.clone();
    let tables = u16::from_be_bytes([replacement[4], replacement[5]]) as usize;
    for i in 0..tables {
        let at = 12 + i * 16;
        if &replacement[at..at + 4] == b"hmtx" {
            let offset =
                u32::from_be_bytes(replacement[at + 8..at + 12].try_into().unwrap()) as usize;
            replacement[offset + 4..offset + 6].copy_from_slice(&600u16.to_be_bytes());
        }
    }
    std::fs::write(dir.path().join("body.font"), &replacement).unwrap();
    manifest.entries[0].resource.font.sha256 = digest(&replacement);
    save(dir.path(), &manifest);
    let second = load(&root);
    assert!(renderer.replace(second.clone()).unwrap());
    assert_eq!(renderer.cached_bindings(), 0);
    assert_eq!(renderer.invalidations(), 1);
    assert!(matches!(
        renderer.verify_replay(&body, &bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
    assert!(renderer
        .bind(&selection("body"), first.generation())
        .is_err());
    let new = renderer
        .bind(&selection("body"), second.generation())
        .unwrap();
    let changed = frame(&renderer, &new);
    assert_ne!(changed.run().advance(), held.run().advance());
    assert_eq!(held.replay_bytes(100000).unwrap(), bytes);
    // Returning to A's generation does not revive A's old epoch lease.
    renderer.replace(first.clone()).unwrap();
    assert!(matches!(
        renderer.verify_replay(&body, &bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
    let rebound = renderer
        .bind(&selection("body"), first.generation())
        .unwrap();
    assert_eq!(
        frame(&renderer, &rebound).replay_bytes(100000).unwrap(),
        bytes
    );
    let other_renderer = RegistryRenderer::new(
        "project-instance",
        first,
        RegistryRenderLimits {
            max_bindings: 2,
            max_cache_bytes: 100000,
        },
    )
    .unwrap();
    assert!(matches!(
        other_renderer.verify_replay(&rebound, &bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
}
#[test]
fn explicit_selection_license_and_replay_identity_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let bytes = font_fixture::shaping_fixture();
    let mut manifest = RegistryManifest {
        schema_version: 1,
        entries: vec![entry(&bytes, "body", "static-truetype", b"test")],
    };
    std::fs::write(dir.path().join("body.font"), bytes).unwrap();
    std::fs::write(dir.path().join("body.license"), b"test").unwrap();
    save(dir.path(), &manifest);
    let first = load(&root);
    let mut renderer = RegistryRenderer::new(
        "project",
        first.clone(),
        RegistryRenderLimits {
            max_bindings: 2,
            max_cache_bytes: 100000,
        },
    )
    .unwrap();
    assert!(renderer
        .bind(&selection("missing"), first.generation())
        .is_err());
    let mut italic = selection("body");
    italic.style = FontStyle::Italic;
    assert!(renderer.bind(&italic, first.generation()).is_err());
    let lease = renderer
        .bind(&selection("body"), first.generation())
        .unwrap();
    let frame = frame(&renderer, &lease);
    let bytes = frame.replay_bytes(100000).unwrap();
    let snapshot = SourceSnapshot {
        revision: 1,
        text: "AA".into(),
    };
    for pointer in [
        "/binding/declaration/license/source",
        "/binding/declaration/license/text_sha256",
        "/binding/declaration/path",
        "/shape/identity/engine_font_id",
    ] {
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        *v.pointer_mut(pointer).unwrap() = serde_json::json!("f".repeat(64));
        assert!(renderer
            .verify_replay(
                &lease,
                &serde_json::to_vec(&v).unwrap(),
                "main.tex",
                &snapshot
            )
            .is_err());
    }
    assert!(frame.replay_bytes(100).is_err());
    manifest.entries[0].resource.license.source = "updated provenance".into();
    save(dir.path(), &manifest);
    assert!(renderer.replace(load(&root)).unwrap());
    assert!(matches!(
        renderer.verify_replay(&lease, &bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
}
#[test]
#[ignore = "requires pinned installed STIX/Liberation fonts and licenses; reads only into a temporary project"]
fn pinned_mixed_backend_registry_replay_and_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let mut entries = Vec::new();
    for (name, format, font_path, license_path, font_sha, license_sha) in [
        (
            "stix",
            "static-cff",
            "/usr/share/fonts/stix-fonts/STIXTwoText-Regular.otf",
            "/usr/share/licenses/stix-fonts/OFL.txt",
            "c4864ca6ec071c2d31d0d8309001faa1ee3517fffb53a31a405a697b71f52ca1",
            "0c8825913b60d858aacdb33c4ca6660a7d64b0d6464702efbb19313f5765861a",
        ),
        (
            "liberation",
            "static-truetype",
            "/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf",
            "/usr/share/licenses/liberation-sans-fonts/LICENSE",
            "76d04c18ea243f426b7de1f3ad208e927008f961dc5945e5aad352d0dfde8ee8",
            "93fed46019c38bbe566b479d22148e2e8a1e85ada614accb0211c37b2c61c19b",
        ),
    ] {
        let font = std::fs::read(font_path).unwrap();
        let license = std::fs::read(license_path).unwrap();
        assert_eq!(digest(&font), font_sha);
        assert_eq!(digest(&license), license_sha);
        let e = entry(&font, name, format, &license);
        std::fs::write(dir.path().join(&e.resource.path), font).unwrap();
        std::fs::write(dir.path().join(&e.resource.license.text_path), license).unwrap();
        entries.push(e);
    }
    let mut manifest = RegistryManifest {
        schema_version: 1,
        entries,
    };
    save(dir.path(), &manifest);
    let registry = load(&root);
    let mut renderer = RegistryRenderer::new(
        "pinned-mixed",
        registry.clone(),
        RegistryRenderLimits {
            max_bindings: 2,
            max_cache_bytes: 2 * 1024 * 1024,
        },
    )
    .unwrap();
    let stix = renderer
        .bind(&selection("stix"), registry.generation())
        .unwrap();
    let liberation = renderer
        .bind(&selection("liberation"), registry.generation())
        .unwrap();
    let s = frame(&renderer, &stix);
    let l = frame(&renderer, &liberation);
    assert!(s.binding().cff_table.is_some());
    assert!(l.binding().cff_table.is_none());
    assert_ne!(s.run().advance(), l.run().advance());
    let snapshot = SourceSnapshot {
        revision: 1,
        text: "AA".into(),
    };
    let s_bytes = s.replay_bytes(1024 * 1024).unwrap();
    let l_bytes = l.replay_bytes(1024 * 1024).unwrap();
    renderer
        .verify_replay(&stix, &s_bytes, "main.tex", &snapshot)
        .unwrap();
    renderer
        .verify_replay(&liberation, &l_bytes, "main.tex", &snapshot)
        .unwrap();
    assert_eq!(
        frame(&renderer, &stix).replay_bytes(1024 * 1024).unwrap(),
        s_bytes
    );
    assert_eq!(
        frame(&renderer, &liberation)
            .replay_bytes(1024 * 1024)
            .unwrap(),
        l_bytes
    );
    let mut wrong: serde_json::Value = serde_json::from_slice(&s_bytes).unwrap();
    wrong["shape"]["primitives"][0]["cff"]["resource"]["cff_sha256"] =
        serde_json::json!("f".repeat(64));
    assert!(renderer
        .verify_replay(
            &stix,
            &serde_json::to_vec(&wrong).unwrap(),
            "main.tex",
            &snapshot
        )
        .is_err());
    // A real ligature retains one indivisible two-byte source cluster.
    use flashtex_rendering_core::registry_binding::selection::*;
    let ligature_source = SourceSnapshot {
        revision: 7,
        text: "fi".into(),
    };
    let ligature_run = renderer
        .shape(
            &liberation,
            ShapeRequest {
                source: &ligature_source.text,
                source_sha256: &digest(ligature_source.text.as_bytes()),
                path: "main.tex",
                revision: 7,
                range: 0..2,
                font_sha256: &liberation.binding().declaration.font.sha256,
                face_index: 0,
                encoding: InputEncoding::Unicode,
                variation_coordinates: &[],
                options: ShapeOptions::default(),
            },
        )
        .unwrap();
    assert_eq!(ligature_run.shaped().clusters.len(), 1);
    let ligature_frame = renderer
        .place(
            &liberation,
            &ligature_run,
            &ligature_source,
            placement(),
            limits(),
            HintPolicy::Unhinted,
        )
        .unwrap();
    let index = renderer
        .selection(
            &liberation,
            &ligature_frame,
            "main.tex",
            &ligature_source,
            vec![cluster_geometry(0, true)],
            SelectionDirection::LeftToRight,
        )
        .unwrap();
    let hit = index
        .inspect(OutlinePoint {
            x: r(9, 1),
            y: r(5, 1),
        })
        .unwrap()
        .unwrap();
    assert_eq!(hit.cluster.source_range, 0..2);
    assert_eq!(hit.caret_byte, 2);
    assert_eq!(hit.cluster.glyph_count, 1);
    // A provenance-only change also changes the global registry generation.
    manifest.entries[0].resource.license.source = "updated explicit provenance".into();
    save(dir.path(), &manifest);
    renderer.replace(load(&root)).unwrap();
    assert_eq!(renderer.cached_bindings(), 0);
    assert!(matches!(
        renderer.verify_replay(&stix, &s_bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
    assert!(matches!(
        renderer.verify_replay(&liberation, &l_bytes, "main.tex", &snapshot),
        Err(BindingError::StaleLease)
    ));
    assert_eq!(s.replay_bytes(1024 * 1024).unwrap(), s_bytes);
    assert_eq!(l.replay_bytes(1024 * 1024).unwrap(), l_bytes);
    println!("STIX replay SHA={} commands={} Liberation replay SHA={} commands={} immutable_retained=true native_painted=false",digest(&s_bytes),s.run().command_count(),digest(&l_bytes),l.run().command_count());
}
fn cluster_geometry(
    index: usize,
    bounds: bool,
) -> flashtex_rendering_core::registry_binding::selection::ClusterGeometry {
    use flashtex_rendering_core::registry_binding::selection::*;
    let x = index as i128 * 10;
    ClusterGeometry {
        cluster_index: index,
        bounds: bounds.then_some(ExactClip {
            left: r(x, 1),
            top: r(0, 1),
            right: r(x + 10, 1),
            bottom: r(10, 1),
        }),
        carets: ClusterCarets {
            start: OutlinePoint {
                x: r(x, 1),
                y: r(5, 1),
            },
            end: OutlinePoint {
                x: r(x + 10, 1),
                y: r(5, 1),
            },
        },
    }
}
#[test]
fn source_aware_selection_preserves_empty_clusters_and_rejects_stale_edits() {
    use flashtex_rendering_core::registry_binding::selection::*;
    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let font = font_fixture::shaping_fixture();
    let mut manifest = RegistryManifest {
        schema_version: 1,
        entries: vec![entry(&font, "body", "static-truetype", b"test")],
    };
    std::fs::write(dir.path().join("body.font"), &font).unwrap();
    std::fs::write(dir.path().join("body.license"), b"test").unwrap();
    save(dir.path(), &manifest);
    let first = load(&root);
    let mut renderer = RegistryRenderer::new(
        "selection",
        first.clone(),
        RegistryRenderLimits {
            max_bindings: 2,
            max_cache_bytes: 100000,
        },
    )
    .unwrap();
    let lease = renderer
        .bind(&selection("body"), first.generation())
        .unwrap();
    let snapshot = SourceSnapshot {
        revision: 2,
        text: "Aé\u{200b}A".into(),
    };
    let run = renderer
        .shape(
            &lease,
            ShapeRequest {
                source: &snapshot.text,
                source_sha256: &digest(snapshot.text.as_bytes()),
                path: "main.tex",
                revision: 2,
                range: 0..snapshot.text.len(),
                font_sha256: &lease.binding().declaration.font.sha256,
                face_index: 0,
                encoding: InputEncoding::Unicode,
                variation_coordinates: &[],
                options: ShapeOptions::PLAIN,
            },
        )
        .unwrap();
    let frame = renderer
        .place(
            &lease,
            &run,
            &snapshot,
            placement(),
            limits(),
            HintPolicy::Unhinted,
        )
        .unwrap();
    let geometry = || {
        vec![
            cluster_geometry(0, true),
            cluster_geometry(1, true),
            cluster_geometry(2, false),
            cluster_geometry(3, true),
        ]
    };
    let index = renderer
        .selection(
            &lease,
            &frame,
            "main.tex",
            &snapshot,
            geometry(),
            SelectionDirection::LeftToRight,
        )
        .unwrap();
    let point = OutlinePoint {
        x: r(18, 1),
        y: r(5, 1),
    };
    let hit = index.inspect(point).unwrap().unwrap();
    assert_eq!(hit.cluster.source_range, 1..3);
    assert_eq!(hit.caret_byte, 3);
    let empty = index.cluster(2).unwrap();
    assert_eq!(empty.glyph_count, 0);
    assert_eq!(empty.source_range, 3..6);
    assert_eq!(empty.text, "\u{200b}");
    assert!(index
        .inspect(OutlinePoint {
            x: r(25, 1),
            y: r(5, 1)
        })
        .unwrap()
        .is_none());
    let destination = renderer
        .destination(&lease, &index, point, "main.tex", &snapshot)
        .unwrap()
        .unwrap();
    assert_eq!(
        renderer
            .validate_destination(&lease, &destination, "main.tex", &snapshot)
            .unwrap(),
        3
    );
    let stale = SourceSnapshot {
        revision: 3,
        ..snapshot.clone()
    };
    assert!(renderer
        .destination(&lease, &index, point, "main.tex", &stale)
        .is_err());
    assert!(renderer
        .validate_destination(&lease, &destination, "main.tex", &stale)
        .is_err());
    assert!(renderer
        .validate_destination(&lease, &destination, "other.tex", &snapshot)
        .is_err());
    assert!(renderer
        .selection(
            &lease,
            &frame,
            "main.tex",
            &snapshot,
            geometry(),
            SelectionDirection::RightToLeft
        )
        .is_err());
    let mut duplicate = geometry();
    duplicate[1].cluster_index = 0;
    assert!(renderer
        .selection(
            &lease,
            &frame,
            "main.tex",
            &snapshot,
            duplicate,
            SelectionDirection::LeftToRight
        )
        .is_err());
    manifest.entries[0].resource.license.source = "new provenance".into();
    save(dir.path(), &manifest);
    let updated = load(&root);
    renderer.replace(updated.clone()).unwrap();
    let current = renderer
        .bind(&selection("body"), updated.generation())
        .unwrap();
    assert_eq!(index.inspect(point).unwrap().unwrap(), hit);
    assert!(renderer
        .destination(&current, &index, point, "main.tex", &snapshot)
        .is_err());
    assert!(renderer
        .validate_destination(&current, &destination, "main.tex", &snapshot)
        .is_err());
    assert!(renderer
        .selection(
            &current,
            &frame,
            "main.tex",
            &snapshot,
            geometry(),
            SelectionDirection::LeftToRight
        )
        .is_err());
}
