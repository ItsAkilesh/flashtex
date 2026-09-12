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
