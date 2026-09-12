use flashtex_font_resources::*;
use std::collections::BTreeMap;

fn be16(data: &mut [u8], at: usize, n: u16) {
    data[at..at + 2].copy_from_slice(&n.to_be_bytes());
}
fn be32(data: &mut [u8], at: usize, n: u32) {
    data[at..at + 4].copy_from_slice(&n.to_be_bytes());
}
fn fixture() -> Vec<u8> {
    let mut tables = BTreeMap::new();
    let mut head = vec![0; 54];
    be32(&mut head, 0, 0x00010000);
    be32(&mut head, 12, 0x5f0f3cf5);
    be16(&mut head, 18, 1000);
    tables.insert(*b"head", head);
    let mut maxp = vec![0; 32];
    be32(&mut maxp, 0, 0x00010000);
    be16(&mut maxp, 4, 3);
    tables.insert(*b"maxp", maxp);
    tables.insert(*b"loca", vec![0; 8]);
    tables.insert(*b"glyf", vec![]);
    let mut hhea = vec![0; 36];
    be32(&mut hhea, 0, 0x00010000);
    be16(&mut hhea, 34, 2);
    tables.insert(*b"hhea", hhea);
    tables.insert(*b"hmtx", vec![0; 10]);
    let mut name = vec![0; 18];
    be16(&mut name, 2, 1);
    be16(&mut name, 4, 18);
    be16(&mut name, 6, 3);
    be16(&mut name, 8, 1);
    be16(&mut name, 10, 0x0409);
    be16(&mut name, 12, 6);
    be16(&mut name, 14, 12);
    for ch in "FTTest".encode_utf16() {
        name.extend_from_slice(&ch.to_be_bytes());
    }
    tables.insert(*b"name", name);
    let mut bytes = vec![0; 12 + tables.len() * 16];
    be32(&mut bytes, 0, 0x00010000);
    be16(&mut bytes, 4, tables.len() as u16);
    for (index, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let start = bytes.len();
        let at = 12 + index * 16;
        bytes[at..at + 4].copy_from_slice(&tag);
        be32(&mut bytes, at + 8, start as u32);
        be32(&mut bytes, at + 12, data.len() as u32);
        bytes.extend_from_slice(&data);
    }
    bytes
}
fn entry(bytes: &[u8]) -> ManifestEntry {
    ManifestEntry {
        font: FontDescriptor {
            font_id: "test.font".into(),
            sha256: sha256(bytes),
            byte_length: bytes.len() as u64,
            format: "static-truetype".into(),
            face_index: 0,
            units_per_em: 1000,
            glyph_count: 3,
            postscript_name: "FTTest".into(),
        },
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Synthetic-Test".into(),
            copyright: "Synthetic fixture only".into(),
            source: "locally generated validation fixture".into(),
            text_path: "LICENSE.txt".into(),
            text_sha256: sha256(b"test license"),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    }
}
fn table_record(bytes: &[u8], tag: &[u8; 4]) -> usize {
    (0..u16::from_be_bytes([bytes[4], bytes[5]]) as usize)
        .map(|i| 12 + i * 16)
        .find(|i| &bytes[*i..*i + 4] == tag)
        .unwrap()
}
fn table_offset(bytes: &[u8], tag: &[u8; 4]) -> usize {
    let at = table_record(bytes, tag) + 8;
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

#[test]
fn immutable_digest_bound_bytes_and_exact_descriptor() {
    let mut bytes = fixture();
    let spec = entry(&bytes);
    let font = FontResource::from_bytes(&spec, &bytes, b"test license").unwrap();
    let copied = font.shared_bytes();
    bytes[0] = 99;
    assert_eq!(font.bytes()[0], 0);
    assert_eq!(&*copied, font.bytes());
    assert_eq!(font.descriptor(), &spec.font);
    assert_eq!(font.table(b"head").unwrap().len(), 54);
    assert_eq!(font.license_text(), b"test license");
    assert_eq!(
        font.license().embedding_permission,
        EmbeddingPermission::Unknown
    );
}
#[test]
fn mismatched_digest_license_and_metadata_rejected() {
    let bytes = fixture();
    let mut spec = entry(&bytes);
    spec.font.sha256 = "0".repeat(64);
    assert_eq!(
        FontResource::from_bytes(&spec, &bytes, b"test license").unwrap_err(),
        Error::DigestMismatch
    );
    let spec = entry(&bytes);
    assert_eq!(
        FontResource::from_bytes(&spec, &bytes, b"changed").unwrap_err(),
        Error::LicenseDigestMismatch
    );
    for field in [
        "units_per_em",
        "glyph_count",
        "postscript_name",
        "byte_length",
    ] {
        let mut spec = entry(&bytes);
        match field {
            "units_per_em" => spec.font.units_per_em = 2048,
            "glyph_count" => spec.font.glyph_count = 4,
            "postscript_name" => spec.font.postscript_name = "Wrong".into(),
            _ => spec.font.byte_length += 1,
        }
        assert_eq!(
            FontResource::from_bytes(&spec, &bytes, b"test license").unwrap_err(),
            Error::MetadataMismatch(field)
        );
    }
}
#[test]
fn face_collection_and_variable_fonts_do_not_fallback() {
    let bytes = fixture();
    let mut spec = entry(&bytes);
    spec.font.face_index = 1;
    assert!(matches!(
        FontResource::from_bytes(&spec, &bytes, b"test license"),
        Err(Error::UnsupportedFont(_))
    ));
    for magic in [b"ttcf", b"OTTO", b"wOFF"] {
        let mut b = fixture();
        b[..4].copy_from_slice(magic);
        assert!(matches!(
            inspect_static_truetype(&b),
            Err(Error::UnsupportedFont(_))
        ));
    }
    let mut b = fixture();
    let at = table_record(&b, b"hmtx");
    b[at..at + 4].copy_from_slice(b"fvar");
    assert!(matches!(
        inspect_static_truetype(&b),
        Err(Error::UnsupportedFont(_))
    ));
}
#[test]
fn directory_bounds_duplicate_and_overlap_rejected() {
    let mut b = fixture();
    be32(&mut b, 20, u32::MAX);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let tag = b[12..16].to_vec();
    b[28..32].copy_from_slice(&tag);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let head = table_offset(&b, b"head");
    let at = table_record(&b, b"hhea");
    be32(&mut b, at + 8, head as u32);
    assert!(inspect_static_truetype(&b).is_err());
}
#[test]
fn loca_and_horizontal_metrics_bounds_rejected() {
    let mut b = fixture();
    let loca = table_offset(&b, b"loca");
    be16(&mut b, loca + 6, 1);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let hhea = table_offset(&b, b"hhea");
    be16(&mut b, hhea + 34, 4);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let record = table_record(&b, b"hmtx");
    be32(&mut b, record + 12, 1);
    assert!(inspect_static_truetype(&b).is_err());
}
#[test]
fn units_glyph_count_name_bounds_checked() {
    let mut b = fixture();
    let head = table_offset(&b, b"head");
    be16(&mut b, head + 18, 0);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let maxp = table_offset(&b, b"maxp");
    be16(&mut b, maxp + 4, 1);
    assert!(inspect_static_truetype(&b).is_err());
    let mut b = fixture();
    let name = table_offset(&b, b"name");
    be16(&mut b, name + 16, 65000);
    assert!(inspect_static_truetype(&b).is_err());
}
#[test]
fn collection_is_sorted_and_lookup_missing_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = fixture();
    std::fs::write(dir.path().join("font.ttf"), &bytes).unwrap();
    std::fs::write(dir.path().join("LICENSE.txt"), b"test license").unwrap();
    let mut a = entry(&bytes);
    a.font.font_id = "a".into();
    let mut z = a.clone();
    z.font.font_id = "z".into();
    let fonts = FontCollection::load(
        dir.path(),
        &Manifest {
            schema_version: 1,
            resources: vec![z, a],
        },
    )
    .unwrap();
    assert_eq!(fonts.ids().collect::<Vec<_>>(), vec!["a", "z"]);
    assert!(matches!(
        fonts.get("fallback"),
        Err(Error::MissingResource(_))
    ));
    std::fs::write(dir.path().join("font.ttf"), b"changed on disk").unwrap();
    assert_eq!(fonts.get("a").unwrap().bytes(), bytes);
}
#[test]
fn missing_files_traversal_and_duplicate_ids_are_errors() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = fixture();
    let spec = entry(&bytes);
    assert!(matches!(
        FontCollection::load(
            dir.path(),
            &Manifest {
                schema_version: 1,
                resources: vec![spec.clone()]
            }
        ),
        Err(Error::MissingResource(_))
    ));
    for path in [
        "../outside.ttf",
        "/absolute.ttf",
        "a//b.ttf",
        "./font.ttf",
        "a\\b.ttf",
    ] {
        let mut s = spec.clone();
        s.path = path.into();
        assert!(matches!(
            FontCollection::load(
                dir.path(),
                &Manifest {
                    schema_version: 1,
                    resources: vec![s]
                }
            ),
            Err(Error::InvalidManifest(_))
        ));
    }
    assert!(matches!(
        FontCollection::load(
            dir.path(),
            &Manifest {
                schema_version: 1,
                resources: vec![spec.clone(), spec]
            }
        ),
        Err(Error::InvalidManifest(_))
    ));
}
#[test]
fn truncated_inputs_never_panic() {
    let bytes = fixture();
    for n in 0..bytes.len() {
        assert!(std::panic::catch_unwind(|| inspect_static_truetype(&bytes[..n])).is_ok());
    }
}
#[test]
fn wire_descriptor_shape_matches_rendering_v2() {
    let value = serde_json::to_value(entry(&fixture()).font).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 8);
    assert_eq!(value["format"], "static-truetype");
    assert_eq!(value["face_index"], 0);
    let mut bad = value;
    bad["platform_fallback"] = serde_json::json!(true);
    assert!(serde_json::from_value::<FontDescriptor>(bad).is_err());
}

#[test]
fn missing_license_is_distinguished_from_missing_font() {
    let root = tempfile::tempdir().unwrap();
    let bytes = fixture();
    std::fs::write(root.path().join("font.ttf"), &bytes).unwrap();
    let manifest = Manifest {
        schema_version: 1,
        resources: vec![entry(&bytes)],
    };
    assert!(matches!(
        FontCollection::load(root.path(), &manifest),
        Err(Error::MissingLicense(_))
    ));
}

#[cfg(unix)]
#[test]
fn symlink_outside_resource_root_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let bytes = fixture();
    std::fs::write(outside.path().join("font.ttf"), &bytes).unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("font.ttf"),
        root.path().join("font.ttf"),
    )
    .unwrap();
    let manifest = Manifest {
        schema_version: 1,
        resources: vec![entry(&bytes)],
    };
    assert!(FontCollection::load(root.path(), &manifest).is_err());
}

#[test]
fn horizontal_metrics_keep_original_units_and_trailing_bearings() {
    let mut bytes = fixture();
    let at = table_record(&bytes, b"hmtx");
    let start = u32::from_be_bytes(bytes[at + 8..at + 12].try_into().unwrap()) as usize;
    be16(&mut bytes, start, 500);
    be16(&mut bytes, start + 4, 700);
    be16(&mut bytes, start + 8, (-30i16) as u16);
    let resource = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    assert_eq!(
        resource.horizontal_metrics(2).unwrap(),
        HorizontalMetrics {
            advance_width: 700,
            left_side_bearing: -30
        }
    );
    assert!(resource.horizontal_metrics(3).is_err());
    assert!(matches!(
        resource.glyph_id('A'),
        Err(Error::UnsupportedFont(_))
    ));
}

fn tfm_for_encoding() -> flashtex_font_resources::tfm::Tfm {
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
    flashtex_font_resources::tfm::Tfm::parse(&bytes).unwrap()
}
fn encoding_manifest(
    font: &FontResource,
    tfm: &flashtex_font_resources::tfm::Tfm,
) -> flashtex_font_resources::encoding::EncodingManifest {
    use flashtex_font_resources::encoding::*;
    EncodingManifest {
        font_sha256: font.descriptor().sha256.clone(),
        tfm_sha256: tfm.source_sha256.clone(),
        face_index: 0,
        encoding: vec![
            EncodingEntry {
                code: 65,
                glyph_name: "A.alt".into(),
            },
            EncodingEntry {
                code: 66,
                glyph_name: ".notdef".into(),
            },
        ],
        declared_glyphs: vec![NamedGlyph {
            glyph_name: "A.alt".into(),
            glyph_id: 2,
        }],
    }
}
#[test]
fn explicit_encoding_never_casts_codes_and_adapter_preserves_metrics() {
    use flashtex_font_resources::encoding::*;
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let bound = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    assert_eq!(bound.map_code(65).unwrap().0, GlyphIdentity::Original(2));
    assert_eq!(bound.map_code(66).unwrap().0, GlyphIdentity::Notdef);
    assert!(bound.map_code(67).is_err());
    assert!(matches!(
        &bound.map_run(b"AB").unwrap()[0],
        MappedItem::Glyph {
            identity: GlyphIdentity::Original(2),
            input_start: 0,
            input_end: 1,
            ..
        }
    ));
}
#[test]
fn encoding_duplicate_missing_and_hash_binding_fail() {
    use flashtex_font_resources::encoding::*;
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let original = encoding_manifest(&font, &tfm);
    for mode in 0..7 {
        let mut m = original.clone();
        match mode {
            0 => m.encoding.push(m.encoding[0].clone()),
            1 => m.declared_glyphs.push(m.declared_glyphs[0].clone()),
            2 => m.declared_glyphs.clear(),
            3 => m.font_sha256 = "0".repeat(64),
            4 => m.tfm_sha256 = "0".repeat(64),
            5 => m.face_index = 1,
            _ => m.declared_glyphs[0].glyph_id = 0,
        };
        assert!(EncodingMap::bind(&m, &font, &tfm).is_err(), "{mode}");
    }
}
#[test]
fn typed_json_escaped_names_decode_exactly_without_postscript_guessing() {
    use flashtex_font_resources::encoding::*;
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let mut manifest = encoding_manifest(&font, &tfm);
    manifest.encoding[0] =
        serde_json::from_str(r#"{"code":65,"glyph_name":"A\u002ealt"}"#).unwrap();
    assert_eq!(
        EncodingMap::bind(&manifest, &font, &tfm)
            .unwrap()
            .resolve(65)
            .unwrap(),
        GlyphIdentity::Original(2)
    );
    manifest.encoding[0].glyph_name = "A#2Ealt".into();
    assert!(EncodingMap::bind(&manifest, &font, &tfm).is_err());
    manifest.declared_glyphs.push(NamedGlyph {
        glyph_name: ".notdef".into(),
        glyph_id: 1,
    });
    assert!(EncodingMap::bind(&manifest, &font, &tfm).is_err());
}

fn virtual_font(commands: &[u8]) -> flashtex_font_resources::vf::VirtualFont {
    let mut b = vec![247, 202, 0];
    for n in [0u32, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([243, 0]);
    for n in [0u32, 1 << 20, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([0, 1, b'f', 242]);
    for n in [commands.len() as u32, 65, 1 << 19] {
        b.extend(n.to_be_bytes());
    }
    b.extend(commands);
    b.push(248);
    flashtex_font_resources::vf::VirtualFont::parse(&b).unwrap()
}
#[test]
fn virtual_packet_expands_exact_bound_glyphs_and_restores_position() {
    use flashtex_font_resources::{encoding::*, vf::Placement};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let binding = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    let resources = BTreeMap::from([(0, binding)]);
    let vf = virtual_font(&[65, 141, 146, 0, 8, 0, 0, 65, 142, 65]);
    let out = vf.expand_packet(65, &tfm, &resources).unwrap();
    let xs = out
        .placements
        .iter()
        .map(|p| match p {
            Placement::Glyph { x, glyph_id, .. } => {
                assert_eq!(*glyph_id, 2);
                (x.numerator(), x.shift())
            }
            _ => panic!("unexpected rule"),
        })
        .collect::<Vec<_>>();
    assert_eq!(xs, vec![(0, 0), (1, 0), (1, 1)]);
}
#[test]
fn virtual_packet_missing_resources_specials_notdef_and_width_fail() {
    use flashtex_font_resources::encoding::*;
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let resources = BTreeMap::from([(0, BoundTfmFont::new(&tfm, &font, &manifest).unwrap())]);
    assert!(virtual_font(&[65])
        .expand_packet(65, &tfm, &BTreeMap::new())
        .is_err());
    assert!(virtual_font(&[66])
        .expand_packet(65, &tfm, &resources)
        .is_err());
    assert!(matches!(
        virtual_font(&[239, 1, 0]).expand_packet(65, &tfm, &resources),
        Err(Error::UnsupportedFont(_))
    ));
    let mut mismatch = tfm.clone();
    mismatch.design_size = flashtex_font_resources::tfm::FixWord(11 << 20);
    assert!(virtual_font(&[65])
        .expand_packet(65, &mismatch, &resources)
        .is_err());
}

#[test]
fn virtual_packet_put_rule_and_reused_register_have_exact_units() {
    use flashtex_font_resources::{encoding::*, vf::Placement};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let resources = BTreeMap::from([(0, BoundTfmFont::new(&tfm, &font, &manifest).unwrap())]);
    let out = virtual_font(&[
        133, 65, 151, 0, 8, 0, 0, 65, 147, 132, 0, 4, 0, 0, 0, 8, 0, 0, 65,
    ])
    .expand_packet(65, &tfm, &resources)
    .unwrap();
    assert_eq!(out.placements.len(), 4);
    match &out.placements[2] {
        Placement::Rule {
            x, width, height, ..
        } => {
            assert_eq!((x.numerator(), x.shift()), (3, 1));
            assert_eq!((width.numerator(), width.shift()), (1, 1));
            assert_eq!((height.numerator(), height.shift()), (1, 2));
        }
        _ => panic!("expected rule"),
    }
    match &out.placements[3] {
        Placement::Glyph { x, .. } => assert_eq!((x.numerator(), x.shift()), (2, 0)),
        _ => panic!("expected glyph"),
    }
}

fn graph_vf(commands: &[u8], scale: u32, comment: u8) -> flashtex_font_resources::vf::VirtualFont {
    let mut b = vec![247, 202, 1, comment];
    for n in [0u32, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([243, 0]);
    for n in [0u32, scale, 10 << 20] {
        b.extend(n.to_be_bytes());
    }
    b.extend([0, 1, b'f', 242]);
    for n in [commands.len() as u32, 65, 1 << 19] {
        b.extend(n.to_be_bytes());
    }
    b.extend(commands);
    b.push(248);
    flashtex_font_resources::vf::VirtualFont::parse(&b).unwrap()
}
#[test]
fn nested_and_flat_vf_geometry_are_exactly_equivalent_with_distinct_provenance() {
    use flashtex_font_resources::{encoding::*, vf_graph::*};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let binding = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    let child = graph_vf(
        &[146, 0, 4, 0, 0, 65, 137, 0, 8, 0, 0, 0, 4, 0, 0],
        1 << 19,
        b'c',
    );
    let root = graph_vf(&[146, 0, 8, 0, 0, 65], 1 << 19, b'r');
    let flat = graph_vf(
        &[146, 0, 10, 0, 0, 65, 137, 0, 4, 0, 0, 0, 2, 0, 0],
        1 << 18,
        b'f',
    );
    let mut graph = ResourceGraph::new();
    let physical = graph.insert(Resource::Physical(&binding)).unwrap();
    let child_key = graph
        .insert(Resource::Virtual {
            vf: &child,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, physical.clone())]),
        })
        .unwrap();
    let root_key = graph
        .insert(Resource::Virtual {
            vf: &root,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, child_key)]),
        })
        .unwrap();
    let flat_key = graph
        .insert(Resource::Virtual {
            vf: &flat,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, physical)]),
        })
        .unwrap();
    let nested = graph.expand(&root_key, 65).unwrap();
    let direct = graph.expand(&flat_key, 65).unwrap();
    let geometry = |p: &NestedPlacement| match p {
        NestedPlacement::Glyph {
            resource,
            glyph_id,
            x,
            y,
            scale,
            ..
        } => (resource.clone(), *glyph_id, *x, *y, *scale),
        _ => panic!("glyph expected"),
    };
    assert_eq!(
        geometry(&nested.placements[0]),
        geometry(&direct.placements[0])
    );
    let rule_geometry = |p: &NestedPlacement| match p {
        NestedPlacement::Rule {
            x,
            y,
            width,
            height,
            ..
        } => (*x, *y, *width, *height),
        _ => panic!("expected rule"),
    };
    assert_eq!(
        rule_geometry(&nested.placements[1]),
        rule_geometry(&direct.placements[1])
    );
    match &nested.placements[0] {
        NestedPlacement::Glyph {
            x, scale, source, ..
        } => {
            assert_eq!((x.numerator(), x.shift()), (5, 3));
            assert_eq!((scale.numerator(), scale.shift()), (1, 2));
            assert_eq!(source.len(), 3);
            assert_eq!(source[0].command_index, Some(1));
        }
        _ => unreachable!(),
    }
}
#[test]
fn nested_vf_cycles_missing_keys_and_specials_fail() {
    use flashtex_font_resources::{encoding::*, vf_graph::*};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let binding = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    let root = graph_vf(&[65], 1 << 20, b'r');
    let key = ResourceKey::Virtual {
        vf_sha256: root.source_sha256.clone(),
        tfm_sha256: tfm.source_sha256.clone(),
    };
    let mut graph = ResourceGraph::new();
    graph
        .insert(Resource::Virtual {
            vf: &root,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, key.clone())]),
        })
        .unwrap();
    assert!(graph.expand(&key, 65).is_err());
    let mut graph = ResourceGraph::new();
    assert!(graph.expand(&key, 65).is_err());
    let physical = graph.insert(Resource::Physical(&binding)).unwrap();
    assert!(graph.insert(Resource::Physical(&binding)).is_err());
    let special = graph_vf(&[239, 1, 0], 1 << 20, b's');
    let key = graph
        .insert(Resource::Virtual {
            vf: &special,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, physical)]),
        })
        .unwrap();
    assert!(matches!(
        graph.expand(&key, 65),
        Err(Error::UnsupportedFont(_))
    ));
}
#[test]
fn nested_vf_depth_and_node_caps_fail_without_partial_output() {
    use flashtex_font_resources::{encoding::*, vf_graph::*};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let binding = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    for branch in [false, true] {
        let fonts = (0..34)
            .map(|i| graph_vf(if branch { &[65, 65] } else { &[65] }, 1 << 20, i))
            .collect::<Vec<_>>();
        let mut graph = ResourceGraph::new();
        let mut key = graph.insert(Resource::Physical(&binding)).unwrap();
        for vf in &fonts {
            key = graph
                .insert(Resource::Virtual {
                    vf,
                    tfm: &tfm,
                    fonts: BTreeMap::from([(0, key)]),
                })
                .unwrap();
            if branch && vf.comment[0] == 12 {
                break;
            }
        }
        assert!(graph.expand(&key, 65).is_err());
    }
}

#[test]
fn nested_vf_output_cap_is_global_across_packets() {
    use flashtex_font_resources::{encoding::*, vf_graph::*};
    let bytes = fixture();
    let font = FontResource::from_bytes(&entry(&bytes), &bytes, b"test license").unwrap();
    let tfm = tfm_for_encoding();
    let manifest = encoding_manifest(&font, &tfm);
    let binding = BoundTfmFont::new(&tfm, &font, &manifest).unwrap();
    let rule = [137, 0, 0, 0, 1, 0, 0, 0, 1];
    let commands = rule.repeat(60000);
    let child = graph_vf(&commands, 1 << 20, b'c');
    let root = graph_vf(&[65, 65], 1 << 20, b'r');
    let mut graph = ResourceGraph::new();
    let physical = graph.insert(Resource::Physical(&binding)).unwrap();
    let child = graph
        .insert(Resource::Virtual {
            vf: &child,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, physical)]),
        })
        .unwrap();
    let root = graph
        .insert(Resource::Virtual {
            vf: &root,
            tfm: &tfm,
            fonts: BTreeMap::from([(0, child)]),
        })
        .unwrap();
    assert!(graph
        .expand(&root, 65)
        .unwrap_err()
        .to_string()
        .contains("output budget"));
}
