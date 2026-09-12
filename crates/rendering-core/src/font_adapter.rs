//! Adapter to the separately owned, original Rust font-resources loader.
//! This validates real sfnt structure/metadata, not full outline-program safety,
//! shaping or renderer parity. No font discovery or document-controlled I/O.
use crate::*;
use flashtex_font_resources::{FontCollection, FontDescriptor};

#[derive(Debug, Clone, Copy, Default)]
pub struct StaticTrueTypeLoader;
impl FontValidator for StaticTrueTypeLoader {
    fn validate_static_truetype(&self, bytes: &[u8]) -> Result<FontMetadata> {
        let metadata = flashtex_font_resources::inspect_static_truetype(bytes)
            .map_err(|error| ValidationError(format!("font resource validation: {error}")))?;
        Ok(FontMetadata {
            units_per_em: metadata.units_per_em,
            glyph_count: metadata.glyph_count,
        })
    }
}
/// Exact descriptor translation; no font name substitution or new wire fields.
pub fn descriptor(font: &FontResource) -> FontDescriptor {
    FontDescriptor {
        font_id: font.font_id.clone(),
        sha256: font.sha256.clone(),
        byte_length: font.byte_length,
        format: font.format.clone(),
        face_index: font.face_index,
        units_per_em: font.units_per_em,
        glyph_count: font.glyph_count,
        postscript_name: font.postscript_name.clone(),
    }
}
/// Validate with a loader-owned immutable collection whose manifests/licenses were
/// already verified. Each requested descriptor must match exactly, including name
/// and face, before the source/GID checks run. Unknown embedding permission remains
/// unknown in the loader and is not converted into export permission here.
pub fn validate_with_collection(
    list: &DisplayList,
    capabilities: &Capabilities,
    documents: &BTreeMap<String, SourceSnapshot>,
    collection: &FontCollection,
) -> Result<ResourceEvidence> {
    list.validate(capabilities)?;
    for font in &list.fonts {
        let resource = collection
            .get(&font.font_id)
            .map_err(|error| ValidationError(format!("font collection: {error}")))?;
        require(
            resource.descriptor() == &descriptor(font),
            "font collection descriptor mismatch",
        )?;
    }
    // FontCollection already owns verified immutable bytes. Exact descriptor
    // equality avoids copying and reparsing every font for each prepared scene.
    for document in &list.documents {
        let snapshot = documents
            .get(&document.path)
            .ok_or_else(|| ValidationError("missing source snapshot".into()))?;
        require(
            snapshot.revision == document.revision,
            "source revision mismatch",
        )?;
        require(
            snapshot.text.len() as u64 == document.byte_length
                && digest(snapshot.text.as_bytes()) == document.sha256,
            "source digest mismatch",
        )?;
    }
    for page in &list.pages {
        for item in &page.items {
            match item {
                Item::GlyphRun(run) => {
                    for cluster in &run.clusters {
                        if let Some(ranges) = &cluster.sources {
                            validate_source_bytes(ranges, documents)?;
                        }
                    }
                }
                Item::Rule(rule) => {
                    if let Some(ranges) = &rule.sources {
                        validate_source_bytes(ranges, documents)?;
                    }
                }
            }
        }
    }
    for diagnostic in &list.diagnostics {
        validate_source_bytes(&diagnostic.sources, documents)?;
    }
    Ok(ResourceEvidence {
        source_snapshots_verified: true,
        font_resources_verified: true,
        paintable: false,
    })
}
