//! Integration fixture against a REAL consumer of `collection_layout`, not a
//! stand-in: `flashtex_font_resources::registry::collections`.
//!
//! That module states its own contract plainly (its doc comment, verbatim):
//! "No collection-directory parser is implemented here. A trusted
//! complete-layout resolver is REQUIRED: the peer font engine selects faces
//! but does not expose validated layout enumeration." It defines
//! `VerifiedCollectionResolver`, a trait an external implementer supplies,
//! and `CollectionRegistry::load` -- the real, public entry point a caller
//! uses -- accepts one as `&dyn VerifiedCollectionResolver`.
//!
//! font-resources' own test suite exercises that trait
//! (`explicit_collection_resolver_registry_identity_and_bounds` in its
//! tests/resources.rs) but only with a hand-rolled `SyntheticResolver` that
//! returns a literal `CollectionLayout` value -- it never calls into this
//! crate's `collection_layout` at all. So until this file, nothing in the
//! repository actually routed this crate's descriptor through that
//! consumer's validated path; this closes that gap from this crate's side,
//! without editing font-resources.
//!
//! `EngineBackedResolver` below is the real adapter: it calls this crate's
//! own `collection_layout`, then reshapes its `(offset, length)`
//! `TableRange`s into the `Range<usize>`-based shape (plus the explicit
//! `header`/`directory` ranges) `registry::collections::CollectionLayout`
//! requires. Both tests drive it through `CollectionRegistry::load` itself,
//! not through `validate_layout` or `build` directly (those are private to
//! font-resources) -- the exact call shape a real caller uses.

use std::path::Path;

use flashtex_font_engine::truetype::{Outlines, TrueTypeFace};
use flashtex_font_engine::{Face, FontSource, collection_layout};
use flashtex_font_resources::registry::collections::{
    CollectionLayout as ResourceLayout, CollectionRegistry, FaceLayout as ResourceFaceLayout,
    TableRange as ResourceTableRange, VerifiedCollectionResolver,
};
use flashtex_font_resources::registry::{
    FontStyle, RegistryEntry, RegistryError, RegistryLimits, RegistryManifest, StyleBinding,
};
use flashtex_font_resources::{
    EmbeddingPermission, Error as ResourceError, FontDescriptor, LicenseMetadata, ManifestEntry,
    Result as ResourceResult, sha256,
};
use flashtex_project_files::ProjectRoot;

const TIMES_TTC: &str = "/System/Library/Fonts/Times.ttc";

/// Adapts [`collection_layout`] to `VerifiedCollectionResolver`.
///
/// `CollectionRegistry`'s only caller of `complete_layout` (its private
/// `build` function) first checks `bytes.starts_with(b"ttcf")`, so only the
/// `ttcf` branch of `collection_layout` is ever reachable through the real
/// consumer; the `header` computed here (the `ttcf` tag/version/offset-table
/// span) reflects that. A bare, non-collection `sfnt` has no such span
/// separate from its one face's own directory, so this adapter is not asked
/// to lay one out through this path -- `EngineBackedResolver` is specific to
/// the collection case font-resources' registry actually uses.
struct EngineBackedResolver;

impl VerifiedCollectionResolver for EngineBackedResolver {
    fn implementation_identity(&self) -> &str {
        "flashtex-font-engine::collection_layout-v1"
    }

    fn complete_layout(&self, bytes: &[u8]) -> ResourceResult<ResourceLayout> {
        let layout = collection_layout(bytes)
            .map_err(|e| ResourceError::InvalidFont(format!("collection_layout: {e}")))?;
        let n = u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let header = 0..12 + 4 * n;
        let faces = layout
            .faces
            .into_iter()
            .map(|f| ResourceFaceLayout {
                directory: f.sfnt_offset..f.sfnt_offset + 12 + 16 * f.tables.len(),
                tables: f
                    .tables
                    .into_iter()
                    .map(|t| ResourceTableRange {
                        tag: t.tag,
                        range: t.offset..t.offset + t.length,
                    })
                    .collect(),
            })
            .collect();
        Ok(ResourceLayout { header, faces })
    }
}

fn write_project(
    dir: &Path,
    font_bytes: &[u8],
    font_path: &str,
    entry: ManifestEntry,
    binding: StyleBinding,
) {
    std::fs::write(dir.join(font_path), font_bytes).unwrap();
    std::fs::write(
        dir.join(&entry.license.text_path),
        b"Apple system font, not redistributed",
    )
    .unwrap();
    std::fs::write(
        dir.join("fonts.json"),
        serde_json::to_vec(&RegistryManifest {
            schema_version: 1,
            entries: vec![RegistryEntry {
                binding,
                resource: entry,
            }],
        })
        .unwrap(),
    )
    .unwrap();
}

fn license_metadata() -> LicenseMetadata {
    LicenseMetadata {
        identifier: "LicenseRef-Apple-System-Font".into(),
        copyright: "Apple system font".into(),
        source: "macOS /System/Library/Fonts, not redistributed".into(),
        text_path: "LICENSE.txt".into(),
        text_sha256: sha256(b"Apple system font, not redistributed"),
        embedding_permission: EmbeddingPermission::Unknown,
    }
}

/// `collection_layout`'s output, reshaped by `EngineBackedResolver`, is
/// accepted by the real `CollectionRegistry::load` for a real `.ttc`, and
/// the face it selects matches what this crate's own `parse_with_source`
/// (which `CollectionRegistry::build` also calls, independently, on the
/// same bytes) reports for that face -- i.e. the two ways this crate's own
/// code describes the same face agree, as seen by an actual peer consumer.
#[test]
fn engine_collection_layout_satisfies_the_real_font_resources_registry() {
    if !Path::new(TIMES_TTC).is_file() {
        eprintln!("SKIP: {TIMES_TTC} not present on this machine");
        return;
    }
    let bytes = std::fs::read(TIMES_TTC).unwrap();
    assert!(
        bytes.starts_with(b"ttcf"),
        "fixture must be a real ttcf collection"
    );

    let face_index = 0u32;
    let reference =
        TrueTypeFace::parse_with_source(bytes.clone(), FontSource::Memory { face_index })
            .unwrap_or_else(|e| panic!("{TIMES_TTC} face {face_index} failed to parse: {e}"));
    let format = match reference.outlines() {
        Outlines::Glyf => "collection-truetype",
        Outlines::Cff => "collection-cff",
    };

    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let binding = StyleBinding {
        family: "Consumer Fixture Times".into(),
        weight: 400,
        style: FontStyle::Upright,
    };
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "times-ttc-face0".into(),
            sha256: sha256(&bytes),
            byte_length: bytes.len() as u64,
            format: format.into(),
            face_index,
            units_per_em: u32::from(reference.units_per_em()),
            glyph_count: u32::from(reference.num_glyphs()),
            postscript_name: reference.postscript_name().to_string(),
        },
        path: "times.ttc".into(),
        license: license_metadata(),
    };
    write_project(dir.path(), &bytes, "times.ttc", entry, binding.clone());

    let registry = CollectionRegistry::load(
        &root,
        "fonts.json",
        RegistryLimits::default(),
        &EngineBackedResolver,
    )
    .unwrap_or_else(|e| panic!("real consumer rejected engine-backed layout: {e:?}"));

    let held = registry.get(&binding, registry.generation()).unwrap();
    assert_eq!(held.face().num_glyphs(), reference.num_glyphs());
    assert_eq!(held.face().postscript_name(), reference.postscript_name());
    assert_eq!(held.bytes(), bytes.as_slice());
    // The registry independently derived per-table identities (tag, range,
    // sha256) from the SAME resolver output by cross-checking each table's
    // range against `TrueTypeFace::table()`'s own byte offsets -- this only
    // succeeds if `collection_layout`'s ranges and `parse_with_source`'s
    // ranges agree exactly.
    assert!(!held.tables().is_empty());
    for identity in held.tables() {
        let actual = reference.table(&identity.tag).unwrap();
        assert_eq!(sha256(actual), identity.sha256);
    }
}

/// A ttcf buffer with a real defect -- a zero-length table, exactly the
/// shape rev 4 added a check for in `collection_layout` -- is rejected by
/// `collection_layout` and that rejection propagates all the way through
/// the real `CollectionRegistry::load`, not just through this crate's own
/// tests. I.e. rev 4's hardening is not just internally tested; it now
/// visibly protects a real downstream consumer, measured here rather than
/// asserted.
#[test]
fn engine_collection_layout_rejection_propagates_through_the_real_registry() {
    let mut bytes = vec![0u8; 200];
    bytes[0..4].copy_from_slice(b"ttcf");
    bytes[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    bytes[8..12].copy_from_slice(&1u32.to_be_bytes()); // numFonts = 1
    bytes[12..16].copy_from_slice(&16u32.to_be_bytes()); // face 0 sfnt offset
    // face 0's sfnt directory at offset 16 (12-byte header, then one 16-byte
    // table record at 28..44): version, 1 table ("head") at offset 100,
    // LENGTH 0 -- rejected by face_layout's zero-length check.
    bytes[16..20].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    bytes[20..22].copy_from_slice(&1u16.to_be_bytes());
    bytes[28..32].copy_from_slice(b"head"); // tag
    // bytes[32..36] left as the (unvalidated) checksum, zero.
    bytes[36..40].copy_from_slice(&100u32.to_be_bytes()); // offset
    bytes[40..44].copy_from_slice(&0u32.to_be_bytes()); // length = 0
    assert!(
        collection_layout(&bytes).is_err(),
        "precondition: collection_layout must actually reject this buffer"
    );

    let dir = tempfile::tempdir().unwrap();
    let root = ProjectRoot::open(dir.path()).unwrap();
    let binding = StyleBinding {
        family: "Consumer Fixture Rejection".into(),
        weight: 400,
        style: FontStyle::Upright,
    };
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "malformed-ttc".into(),
            sha256: sha256(&bytes),
            byte_length: bytes.len() as u64,
            format: "collection-truetype".into(),
            face_index: 0,
            units_per_em: 1000,
            glyph_count: 3,
            postscript_name: "Placeholder".into(),
        },
        path: "malformed.ttc".into(),
        license: license_metadata(),
    };
    write_project(dir.path(), &bytes, "malformed.ttc", entry, binding);

    // `CollectionRegistry` does not derive `Debug`, so this is matched by
    // hand rather than via `expect_err`/`unwrap_err`.
    let result = CollectionRegistry::load(
        &root,
        "fonts.json",
        RegistryLimits::default(),
        &EngineBackedResolver,
    );
    let err = match result {
        Ok(_) => panic!("a font_engine-rejected layout must not reach the registry as valid"),
        Err(e) => e,
    };
    match err {
        RegistryError::ResourceMismatch { path, .. } => assert_eq!(path, "malformed.ttc"),
        other => panic!("expected ResourceMismatch, got {other:?}"),
    }
}
