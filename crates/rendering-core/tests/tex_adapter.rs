use flashtex_font_resources::{encoding::*, tfm::*, vf::VirtualFont, *};
use flashtex_rendering_core::{digest, tex_adapter::*, Tick};
use std::collections::BTreeMap;
#[path = "support/font_fixture.rs"]
mod font_fixture;
fn font(triangle: bool) -> FontResource {
    let bytes = if triangle {
        font_fixture::triangle_fixture()
    } else {
        font_fixture::fixture()
    };
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "test".into(),
            sha256: digest(&bytes),
            byte_length: bytes.len() as u64,
            format: "static-truetype".into(),
            face_index: 0,
            units_per_em: 1000,
            glyph_count: 3,
            postscript_name: "FTTest".into(),
        },
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Synthetic".into(),
            copyright: "original".into(),
            source: "test".into(),
            text_path: "LICENSE".into(),
            text_sha256: digest(b"synthetic"),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    FontResource::from_bytes(&entry, &bytes, b"synthetic").unwrap()
}
fn tfm() -> Tfm {
    let mut bytes = [16u16, 2, 65, 66, 2, 1, 1, 1, 0, 0, 0, 1]
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>();
    for word in [
        0u32,
        10 << 20,
        0x01000000,
        0x01000000,
        0,
        1 << 19,
        0,
        0,
        0,
        0,
    ] {
        bytes.extend(word.to_be_bytes());
    }
    Tfm::parse(&bytes).unwrap()
}
fn manifest(font: &FontResource, tfm: &Tfm) -> EncodingManifest {
    EncodingManifest {
        font_sha256: font.descriptor().sha256.clone(),
        tfm_sha256: tfm.source_sha256.clone(),
        face_index: 0,
        encoding: vec![
            EncodingEntry {
                code: 65,
                glyph_name: "triangle".into(),
            },
            EncodingEntry {
                code: 66,
                glyph_name: ".notdef".into(),
            },
        ],
        declared_glyphs: vec![NamedGlyph {
            glyph_name: "triangle".into(),
            glyph_id: 1,
        }],
    }
}
fn vf(commands: &[u8]) -> VirtualFont {
    let mut b = vec![247, 202, 0];
    for n in [0u32, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([243, 0]);
    for n in [0u32, 1 << 20, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([0, 1, b'f']);
    b.extend([commands.len() as u8, 65, 8, 0, 0]);
    b.extend(commands);
    b.push(248);
    VirtualFont::parse(&b).unwrap()
}
fn scale() -> RunScale {
    RunScale::canonical(Tick(1000), MetricPolicy::ExactRationalNoTexRounding).unwrap()
}
#[test]
fn encoded_slot_is_not_gid_and_intervals_survive() {
    let f = font(true);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let run = physical_run(&b, b"AA", scale()).unwrap();
    assert_eq!(run.advance.require_integer().unwrap(), Tick(1000));
    let Operation::Glyph(g) = &run.operations[1] else {
        panic!()
    };
    assert_eq!(g.original_gid, 1);
    assert_eq!(g.code, 65);
    assert_eq!(g.x.require_integer().unwrap(), Tick(500));
    assert_eq!(g.input, InputInterval { start: 1, end: 2 });
    assert_eq!(g.tfm_sha256, t.source_sha256);
}
#[test]
fn fractional_metric_is_retained_not_rounded() {
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let run = physical_run(
        &b,
        b"A",
        RunScale::canonical(Tick(1), MetricPolicy::ExactRationalNoTexRounding).unwrap(),
    )
    .unwrap();
    assert_eq!(run.advance.numerator(), 1);
    assert_eq!(run.advance.denominator(), 2);
    assert_eq!(
        run.advance.require_integer(),
        Err(AdapterError::NonIntegralTicks)
    );
    let design = physical_run(
        &b,
        b"A",
        RunScale::design_size(t.design_size, MetricPolicy::ExactRationalNoTexRounding).unwrap(),
    )
    .unwrap();
    assert_eq!(
        design.advance.numerator() * 7227,
        ((10i128 << 20) * 3600) * (design.advance.denominator() as i128)
    );
}
#[test]
fn missing_notdef_and_budget_are_explicit() {
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    assert!(matches!(
        physical_run(&b, b"B", scale()),
        Err(AdapterError::UnsupportedNotdef { code: 66 })
    ));
    assert!(matches!(
        physical_run(&b, b"C", scale()),
        Err(AdapterError::Resource(_))
    ));
    assert!(matches!(
        physical_run(&b, &vec![65; 100001], scale()),
        Err(AdapterError::Budget)
    ));
}
#[test]
fn virtual_and_flat_exact_positions_match() {
    let f = font(true);
    let t = tfm();
    let m = manifest(&f, &t);
    let b = BoundTfmFont::new(&t, &f, &m).unwrap();
    let flat = physical_run(&b, b"AA", scale()).unwrap();
    let bindings = BTreeMap::from([(0, b)]);
    let virtual_ = virtual_run(&vf(&[65]), &t, &bindings, b"AA", scale()).unwrap();
    assert_eq!(flat.advance, virtual_.advance);
    assert!(virtual_.vf_sha256.is_some());
    for (a, b) in flat.operations.iter().zip(&virtual_.operations) {
        let (Operation::Glyph(a), Operation::Glyph(b)) = (a, b) else {
            panic!()
        };
        assert_eq!(
            (a.x, a.baseline_y, a.size, a.original_gid, &a.input),
            (b.x, b.baseline_y, b.size, b.original_gid, &b.input)
        );
    }
}
#[test]
fn virtual_special_and_missing_binding_do_not_emit_partial_run() {
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let bindings = BTreeMap::from([(0, b)]);
    assert!(matches!(
        virtual_run(&vf(&[65, 239, 1, b'x']), &t, &bindings, b"A", scale()),
        Err(AdapterError::UnsupportedFont(_))
    ));
    assert!(virtual_run(&vf(&[65]), &t, &BTreeMap::new(), b"A", scale()).is_err());
}
#[test]
fn virtual_rule_lower_left_becomes_top_edge() {
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let bindings = BTreeMap::from([(0, b)]);
    let mut cmd = vec![137];
    cmd.extend((1i32 << 19).to_be_bytes());
    cmd.extend((1i32 << 18).to_be_bytes());
    let run = virtual_run(&vf(&cmd), &t, &bindings, b"A", scale()).unwrap();
    let Operation::Rule {
        x,
        top,
        width,
        height,
        ..
    } = &run.operations[0]
    else {
        panic!()
    };
    assert_eq!(x.require_integer().unwrap(), Tick(0));
    assert_eq!(top.require_integer().unwrap(), Tick(-500));
    assert_eq!(width.require_integer().unwrap(), Tick(250));
    assert_eq!(height.require_integer().unwrap(), Tick(500));
}
#[test]
fn bound_run_emits_existing_paths_with_explicit_utf8_provenance() {
    use flashtex_rendering_core::{batch::*, glyph_cache::GlyphPathCache, hit_test::Point, Paint};
    let f = font(true);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let run = physical_run(&b, b"A", scale()).unwrap();
    let provenance = vec![Provenance {
        input: InputInterval { start: 0, end: 1 },
        logical_start: 0,
        logical_end: 2,
        sources: vec![],
        synthetic_reason: Some("Explicit encoded fixture".into()),
    }];
    let snapshots = BTreeMap::new();
    let ctx = BatchContext {
        project_id: "test",
        revision: 1,
        page: 1,
        page_width: Tick(2000),
        page_height: Tick(2000),
        origin: Point {
            x: Tick(0),
            y: Tick(1000),
        },
        clip: None,
        paint: Paint {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        logical_text: "é",
        provenance: &provenance,
        documents: &[],
        snapshots: &snapshots,
    };
    let mut cache = GlyphPathCache::new(8, 100000).unwrap();
    let batch = run.batch(&ctx, BatchLimits::default(), &mut cache).unwrap();
    let DrawOperation::Glyph { path, .. } = &batch.operations[0] else {
        panic!()
    };
    assert_eq!(path.original_gid, 1);
    assert_eq!(path.logical_end_byte, 2);
    assert_eq!(path.commands.len(), 4);
    assert!(!path.hinting_applied);
    let fractional = physical_run(
        &b,
        b"AA",
        RunScale::canonical(Tick(1), MetricPolicy::ExactRationalNoTexRounding).unwrap(),
    )
    .unwrap();
    let mut ctx = ctx;
    let mappings = vec![
        provenance[0].clone(),
        Provenance {
            input: InputInterval { start: 1, end: 2 },
            ..provenance[0].clone()
        },
    ];
    ctx.provenance = &mappings;
    let fractional_batch = fractional
        .batch(&ctx, BatchLimits::default(), &mut cache)
        .unwrap();
    let DrawOperation::Glyph { path, .. } = &fractional_batch.operations[1] else {
        panic!()
    };
    let flashtex_rendering_core::outlines::PlacedPathCommand::MoveTo(point) = path.commands[0]
    else {
        panic!()
    };
    assert_eq!((point.x.numerator(), point.x.denominator()), (1, 2));
    let frac =
        |n, d| flashtex_rendering_core::outlines::OutlineCoordinate::from_fraction(n, d).unwrap();
    let exact_clip = ExactClip {
        left: frac(1, 4),
        top: frac(999, 1),
        right: frac(3, 4),
        bottom: frac(1001, 1),
    };
    let clipped = fractional
        .batch_with_exact_clip(&ctx, exact_clip, BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(clipped.visible_clip, Some(exact_clip));
    assert_eq!(clipped.batch.operations.len(), 2);
    ctx.provenance = &[];
    assert!(run.batch(&ctx, BatchLimits::default(), &mut cache).is_err());
}
#[test]
fn explicit_source_identity_and_utf8_boundaries_gate_batches() {
    use flashtex_rendering_core::{
        batch::*, glyph_cache::GlyphPathCache, hit_test::Point, DocumentResource, Paint,
        SourceRange, SourceSnapshot,
    };
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let run = physical_run(&b, b"A", scale()).unwrap();
    let mut mapping = vec![Provenance {
        input: InputInterval { start: 0, end: 1 },
        logical_start: 0,
        logical_end: 2,
        sources: vec![SourceRange {
            path: "main.tex".into(),
            start_byte: 0,
            end_byte: 2,
        }],
        synthetic_reason: None,
    }];
    let mut docs = vec![DocumentResource {
        path: "main.tex".into(),
        revision: 1,
        sha256: digest("é".as_bytes()),
        byte_length: 2,
    }];
    let snapshots = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: "é".into(),
        },
    )]);
    let mut cache = GlyphPathCache::new(8, 100000).unwrap();
    let check = |mapping: &[Provenance], docs: &[DocumentResource], cache: &mut GlyphPathCache| {
        run.batch(
            &BatchContext {
                project_id: "test",
                revision: 1,
                page: 1,
                page_width: Tick(2000),
                page_height: Tick(2000),
                origin: Point {
                    x: Tick(0),
                    y: Tick(1000),
                },
                clip: None,
                paint: Paint {
                    r: 0.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                },
                logical_text: "é",
                provenance: mapping,
                documents: docs,
                snapshots: &snapshots,
            },
            BatchLimits::default(),
            cache,
        )
    };
    assert!(check(&mapping, &docs, &mut cache).is_ok());
    mapping[0].sources[0].end_byte = 1;
    assert!(check(&mapping, &docs, &mut cache).is_err());
    mapping[0].sources[0].end_byte = 2;
    docs[0].revision = 2;
    assert!(check(&mapping, &docs, &mut cache).is_err());
}
#[test]
fn graph_cache_is_immutable_scoped_and_preserves_exact_source_chain() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::*;
    let f = font(true);
    let t = tfm();
    let m = manifest(&f, &t);
    let b = BoundTfmFont::new(&t, &f, &m).unwrap();
    let v = vf(&[65]);
    let mut graph = ResourceGraph::new();
    let p = graph.insert(Resource::Physical(&b)).unwrap();
    let root = graph
        .insert(Resource::Virtual {
            vf: &v,
            tfm: &t,
            fonts: BTreeMap::from([(0, p)]),
        })
        .unwrap();
    let mut cache = GraphCache::new(&graph, 4, 10000).unwrap();
    let Outcome::Ready(a) = cache.lookup(&root, 65).unwrap().outcome else {
        panic!()
    };
    let hit = cache.lookup(&root, 65).unwrap();
    assert!(hit.cache_hit);
    let Outcome::Ready(b) = hit.outcome else {
        panic!()
    };
    assert!(std::sync::Arc::ptr_eq(&a, &b));
    let NestedPlacement::Glyph {
        source, glyph_id, ..
    } = &a.placements[0]
    else {
        panic!()
    };
    assert_eq!(*glyph_id, 1);
    assert_eq!(source.len(), 2);
    assert_eq!(cache.stats().expansions, 1);
    cache.clear();
    assert_eq!(cache.stats().entries, 0);
    assert_eq!(a.placements.len(), 1);
}
#[test]
fn graph_cache_negative_outcomes_lru_and_payload_caps() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::*;
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let mut graph = ResourceGraph::new();
    let root = graph.insert(Resource::Physical(&b)).unwrap();
    let mut cache = GraphCache::new(&graph, 1, 1024).unwrap();
    let Outcome::Unavailable(reason) = cache.lookup(&root, 65).unwrap().outcome else {
        panic!()
    };
    assert!(matches!(*reason, Failure::PayloadBudget { .. }));
    assert!(cache.lookup(&root, 65).unwrap().cache_hit);
    let Outcome::Unavailable(reason) = cache.lookup(&root, 66).unwrap().outcome else {
        panic!()
    };
    assert!(matches!(*reason, Failure::Resource(_)));
    assert_eq!(cache.stats().evictions, 1);
    assert!(!cache.lookup(&root, 65).unwrap().cache_hit);
    assert!(cache.stats().retained_payload_bytes <= 1024);
    let invalid = ResourceKey::Physical {
        font_sha256: "invalid".into(),
        tfm_sha256: "0".repeat(64),
        face_index: 0,
    };
    assert!(cache.lookup(&invalid, 65).is_err());
}
#[test]
fn identical_file_hashes_with_different_encoding_do_not_share_cache() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::*;
    let f = font(false);
    let t = tfm();
    let mut m = manifest(&f, &t);
    let a = BoundTfmFont::new(&t, &f, &m).unwrap();
    m.declared_glyphs[0].glyph_id = 2;
    let b = BoundTfmFont::new(&t, &f, &m).unwrap();
    let mut ga = ResourceGraph::new();
    let ka = ga.insert(Resource::Physical(&a)).unwrap();
    let mut gb = ResourceGraph::new();
    let kb = gb.insert(Resource::Physical(&b)).unwrap();
    assert_eq!(ka, kb);
    let mut ca = GraphCache::new(&ga, 4, 10000).unwrap();
    let mut cb = GraphCache::new(&gb, 4, 10000).unwrap();
    let Outcome::Ready(a) = ca.lookup(&ka, 65).unwrap().outcome else {
        panic!()
    };
    let Outcome::Ready(b) = cb.lookup(&kb, 65).unwrap().outcome else {
        panic!()
    };
    let NestedPlacement::Glyph { glyph_id: a, .. } = a.placements[0] else {
        panic!()
    };
    let NestedPlacement::Glyph { glyph_id: b, .. } = b.placements[0] else {
        panic!()
    };
    assert_eq!((a, b), (1, 2));
}
#[test]
fn signed_tfm_kern_moves_following_original_glyph_exactly() {
    let mut bytes = [18u16, 2, 65, 66, 2, 1, 1, 1, 1, 1, 0, 1]
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>();
    for word in [
        0u32,
        10 << 20,
        0x01000100,
        0x01000000,
        0,
        1 << 19,
        0,
        0,
        0,
        0x80428000,
        (-(1i32 << 18)) as u32,
        0,
    ] {
        bytes.extend(word.to_be_bytes());
    }
    let t = Tfm::parse(&bytes).unwrap();
    let f = font(true);
    let mut m = manifest(&f, &t);
    m.encoding[1].glyph_name = "triangle".into();
    let b = BoundTfmFont::new(&t, &f, &m).unwrap();
    let run = physical_run(&b, b"AB", scale()).unwrap();
    let Operation::Glyph(g) = &run.operations[1] else {
        panic!()
    };
    assert_eq!(g.x.require_integer().unwrap(), Tick(250));
    assert_eq!(g.original_gid, 1);
    assert_eq!(g.code, 66);
    assert_eq!(g.input, InputInterval { start: 1, end: 2 });
    assert_eq!(run.advance.require_integer().unwrap(), Tick(750));
}
#[test]
fn fractional_virtual_rule_emits_exact_internal_geometry() {
    use flashtex_rendering_core::{
        batch::*, glyph_cache::GlyphPathCache, hit_test::Point, outlines::OutlineCoordinate, Paint,
    };
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let bindings = BTreeMap::from([(0, b)]);
    let mut cmd = vec![137];
    cmd.extend((1i32 << 19).to_be_bytes());
    cmd.extend((1i32 << 18).to_be_bytes());
    let run = virtual_run(
        &vf(&cmd),
        &t,
        &bindings,
        b"A",
        RunScale::canonical(Tick(1), MetricPolicy::ExactRationalNoTexRounding).unwrap(),
    )
    .unwrap();
    let mappings = [Provenance {
        input: InputInterval { start: 0, end: 1 },
        logical_start: 0,
        logical_end: 1,
        sources: vec![],
        synthetic_reason: Some("fixture rule".into()),
    }];
    let snapshots = BTreeMap::new();
    let ctx = BatchContext {
        project_id: "test",
        revision: 1,
        page: 1,
        page_width: Tick(2),
        page_height: Tick(2),
        origin: Point {
            x: Tick(0),
            y: Tick(1),
        },
        clip: None,
        paint: Paint {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        logical_text: "A",
        provenance: &mappings,
        documents: &[],
        snapshots: &snapshots,
    };
    let mut cache = GlyphPathCache::new(4, 10000).unwrap();
    let batch = run.batch(&ctx, BatchLimits::default(), &mut cache).unwrap();
    let DrawOperation::ExactRule { geometry, .. } = batch.operations[0] else {
        panic!()
    };
    assert_eq!(
        geometry.top,
        OutlineCoordinate::from_fraction(1, 2).unwrap()
    );
    assert_eq!(
        geometry.right,
        OutlineCoordinate::from_fraction(1, 4).unwrap()
    );
    assert_eq!(
        geometry.bottom,
        OutlineCoordinate::from_fraction(1, 1).unwrap()
    );
}
#[test]
fn nested_graph_run_matches_flat_fractional_positions_and_retains_source_chain() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::GraphCache;
    let f = font(true);
    let t = tfm();
    let m = manifest(&f, &t);
    let b = BoundTfmFont::new(&t, &f, &m).unwrap();
    let child = vf(&[65]);
    let root_vf = vf(&[138, 65]);
    let mut graph = ResourceGraph::new();
    let pk = graph.insert(Resource::Physical(&b)).unwrap();
    let ck = graph
        .insert(Resource::Virtual {
            vf: &child,
            tfm: &t,
            fonts: BTreeMap::from([(0, pk.clone())]),
        })
        .unwrap();
    let root = graph
        .insert(Resource::Virtual {
            vf: &root_vf,
            tfm: &t,
            fonts: BTreeMap::from([(0, ck)]),
        })
        .unwrap();
    let physical = BTreeMap::from([(pk, &b)]);
    let mut cache = GraphCache::new(&graph, 8, 100000).unwrap();
    let scale = RunScale::canonical(Tick(1), MetricPolicy::ExactRationalNoTexRounding).unwrap();
    let nested = nested_run(&mut cache, &root, &t, &physical, b"AA", scale).unwrap();
    let flat = physical_run(&b, b"AA", scale).unwrap();
    assert_eq!(nested.run().advance, flat.advance);
    assert_eq!(nested.source_chains().len(), 2);
    assert_eq!(nested.source_chains()[0].len(), 3);
    assert_eq!(nested.source_chains()[0][0].command_index, Some(1));
    for (a, b) in nested.run().operations.iter().zip(&flat.operations) {
        let (Operation::Glyph(a), Operation::Glyph(b)) = (a, b) else {
            panic!()
        };
        assert_eq!(
            (a.x, a.baseline_y, a.size, a.original_gid, &a.input),
            (b.x, b.baseline_y, b.size, b.original_gid, &b.input)
        );
    }
    assert_eq!(cache.stats().expansions, 1);
    assert_eq!(cache.stats().hits, 1);
}
#[test]
fn nested_consumer_rejects_differing_encoding_or_missing_physical_binding() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::GraphCache;
    let f = font(false);
    let t = tfm();
    let mut m = manifest(&f, &t);
    let original = BoundTfmFont::new(&t, &f, &m).unwrap();
    m.declared_glyphs[0].glyph_id = 2;
    let wrong = BoundTfmFont::new(&t, &f, &m).unwrap();
    let mut graph = ResourceGraph::new();
    let root = graph.insert(Resource::Physical(&original)).unwrap();
    let mut cache = GraphCache::new(&graph, 4, 100000).unwrap();
    assert!(matches!(
        nested_run(&mut cache, &root, &t, &BTreeMap::new(), b"A", scale()),
        Err(AdapterError::Resource(_))
    ));
    assert!(matches!(
        nested_run(
            &mut cache,
            &root,
            &t,
            &BTreeMap::from([(root.clone(), &wrong)]),
            b"A",
            scale()
        ),
        Err(AdapterError::Resource(_))
    ));
    let mut changed = t.clone();
    changed.source_sha256 = "0".repeat(64);
    assert!(nested_run(&mut cache, &root, &changed, &BTreeMap::new(), b"A", scale()).is_err());
}
#[test]
fn nested_graph_rules_preserve_fractional_geometry_and_input_interval() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::graph_cache::GraphCache;
    let f = font(false);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let mut commands = vec![137];
    commands.extend((1i32 << 19).to_be_bytes());
    commands.extend((1i32 << 18).to_be_bytes());
    let child = vf(&commands);
    let parent = vf(&[138, 65]);
    let mut graph = ResourceGraph::new();
    let pk = graph.insert(Resource::Physical(&b)).unwrap();
    let ck = graph
        .insert(Resource::Virtual {
            vf: &child,
            tfm: &t,
            fonts: BTreeMap::from([(0, pk.clone())]),
        })
        .unwrap();
    let root = graph
        .insert(Resource::Virtual {
            vf: &parent,
            tfm: &t,
            fonts: BTreeMap::from([(0, ck)]),
        })
        .unwrap();
    let mut cache = GraphCache::new(&graph, 8, 100000).unwrap();
    let result = nested_run(
        &mut cache,
        &root,
        &t,
        &BTreeMap::from([(pk, &b)]),
        b"A",
        RunScale::canonical(Tick(1), MetricPolicy::ExactRationalNoTexRounding).unwrap(),
    )
    .unwrap();
    let Operation::Rule {
        top,
        width,
        height,
        input,
        ..
    } = &result.run().operations[0]
    else {
        panic!()
    };
    assert_eq!((top.numerator(), top.denominator()), (-1, 2));
    assert_eq!((width.numerator(), width.denominator()), (1, 4));
    assert_eq!((height.numerator(), height.denominator()), (1, 2));
    assert_eq!(input, &InputInterval { start: 0, end: 1 });
    assert_eq!(result.source_chains()[0].len(), 2);
}
#[test]
fn traced_nested_batches_keep_source_identity_after_culling_and_reordering() {
    use flashtex_font_resources::vf_graph::*;
    use flashtex_rendering_core::{
        batch::*, glyph_cache::GlyphPathCache, graph_cache::GraphCache, hit_test::Point,
        outlines::OutlineCoordinate, HitRect, Paint,
    };
    let f = font(true);
    let t = tfm();
    let b = BoundTfmFont::new(&t, &f, &manifest(&f, &t)).unwrap();
    let mut rule = vec![137];
    rule.extend((1i32 << 19).to_be_bytes());
    rule.extend((1i32 << 18).to_be_bytes());
    let mut commands = rule.clone();
    commands.push(65);
    commands.extend(rule);
    let virtual_ = vf(&commands);
    let mut graph = ResourceGraph::new();
    let pk = graph.insert(Resource::Physical(&b)).unwrap();
    let root = graph
        .insert(Resource::Virtual {
            vf: &virtual_,
            tfm: &t,
            fonts: BTreeMap::from([(0, pk.clone())]),
        })
        .unwrap();
    let mut graph_cache = GraphCache::new(&graph, 4, 100000).unwrap();
    let nested = nested_run(
        &mut graph_cache,
        &root,
        &t,
        &BTreeMap::from([(pk, &b)]),
        b"A",
        scale(),
    )
    .unwrap();
    let mappings = [Provenance {
        input: InputInterval { start: 0, end: 1 },
        logical_start: 0,
        logical_end: 1,
        sources: vec![],
        synthetic_reason: Some("explicit VF fixture".into()),
    }];
    let snapshots = BTreeMap::new();
    let clip = HitRect {
        x: Tick(300),
        top: Tick(0),
        width: Tick(700),
        height: Tick(2000),
    };
    let ctx = BatchContext {
        project_id: "test",
        revision: 3,
        page: 1,
        page_width: Tick(2000),
        page_height: Tick(2000),
        origin: Point {
            x: Tick(0),
            y: Tick(1000),
        },
        clip: Some(&clip),
        paint: Paint {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        logical_text: "A",
        provenance: &mappings,
        documents: &[],
        snapshots: &snapshots,
    };
    let mut cache = GlyphPathCache::new(4, 100000).unwrap();
    let mut batch = nested
        .batch(&ctx, None, BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(batch.primitives.len(), 2);
    assert_eq!(batch.primitives[0].identity.item_index, 1);
    assert_eq!(batch.primitives[1].identity.item_index, 2);
    for primitive in &batch.primitives {
        assert_eq!(primitive.identity, primitive.operation.primitive_id());
        assert_eq!(
            primitive.source_chain[0].command_index,
            Some(primitive.identity.item_index)
        );
    }
    batch.primitives.reverse();
    assert_eq!(batch.primitives[0].identity.item_index, 2);
    assert_eq!(batch.primitives[0].source_chain[0].command_index, Some(2));
    assert_eq!(batch.primitives[1].source_chain[0].command_index, Some(1));
    let c = |n, d| OutlineCoordinate::from_fraction(n, d).unwrap();
    let exact = ExactClip {
        left: c(601, 2),
        top: c(0, 1),
        right: c(799, 2),
        bottom: c(2000, 1),
    };
    let batch = nested
        .batch(&ctx, Some(exact), BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(batch.primitives.len(), 1);
    assert_eq!(batch.primitives[0].identity.item_index, 1);
    assert_eq!(batch.primitives[0].source_chain[0].command_index, Some(1));
    assert_eq!(batch.visible_clip, Some(exact));
}
