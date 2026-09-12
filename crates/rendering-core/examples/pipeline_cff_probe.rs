//! Opt-in actual pipeline display -> immutable registry CFF -> exact PDF.
//! Args: DISPLAY REQUEST FONT LICENSE OUTPUT_PREFIX. No source rewriting.
use flashtex_font_resources::{cff::HintPolicy, registry::*, *};
use flashtex_project_files::ProjectRoot;
use flashtex_rendering_core::{
    mixed::MixedLimits,
    pdf_export::{export, ExportLimits},
    pdf_stream::{PdfCommandStream, StreamLimits},
    pipeline_cff::PipelineCff,
    *,
};
use std::{collections::BTreeMap, path::Path};
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 6 {
        return Err("DISPLAY REQUEST FONT LICENSE OUTPUT_PREFIX".into());
    }
    let bytes = std::fs::read(&a[1])?;
    let Message::DisplayList(list) = parse(&bytes)?.message else {
        return Err("display required".into());
    };
    if list.fonts.len() != 1 {
        return Err("probe requires one explicit CFF font".into());
    }
    let f = &list.fonts[0];
    let font = std::fs::read(&a[3])?;
    let license = std::fs::read(&a[4])?;
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("font.otf"), &font)?;
    std::fs::write(temp.path().join("LICENSE"), &license)?;
    let binding = StyleBinding {
        family: "Probe".into(),
        weight: 400,
        style: FontStyle::Upright,
    };
    let manifest = RegistryManifest {
        schema_version: 1,
        entries: vec![RegistryEntry {
            binding: binding.clone(),
            resource: ManifestEntry {
                font: FontDescriptor {
                    font_id: f.font_id.clone(),
                    sha256: digest(&font),
                    byte_length: f.byte_length,
                    format: "static-cff".into(),
                    face_index: f.face_index,
                    units_per_em: f.units_per_em,
                    glyph_count: f.glyph_count,
                    postscript_name: f.postscript_name.clone(),
                },
                path: "font.otf".into(),
                license: LicenseMetadata {
                    identifier: "LicenseRef-Explicit".into(),
                    copyright: "See supplied license".into(),
                    source: a[4].clone(),
                    text_path: "LICENSE".into(),
                    text_sha256: digest(&license),
                    embedding_permission: EmbeddingPermission::Unknown,
                },
            },
        }],
    };
    std::fs::write(
        temp.path().join("fonts.json"),
        serde_json::to_vec(&manifest)?,
    )?;
    let root = ProjectRoot::open(temp.path())?;
    let registry = ProjectFontRegistry::load(&root, "fonts.json", RegistryLimits::default())?;
    let RegistryResource::Cff(resource) = registry.resource(&binding)? else {
        return Err("CFF required".into());
    };
    let req: serde_json::Value = serde_json::from_slice(&std::fs::read(&a[2])?)?;
    let mut docs = BTreeMap::new();
    for d in req["payload"]["documents"]
        .as_array()
        .ok_or("documents missing")?
    {
        docs.insert(
            d["path"].as_str().ok_or("path")?.into(),
            SourceSnapshot {
                revision: req["payload"]["revision"].as_u64().ok_or("revision")?,
                text: d["text"].as_str().ok_or("text")?.into(),
            },
        );
    }
    let Message::Offer(caps) =
        parse(include_bytes!("../tests/fixtures/capabilities.json"))?.message
    else {
        unreachable!()
    };
    let adapter = PipelineCff::bind(
        &bytes,
        &caps,
        &docs,
        &BTreeMap::from([(f.font_id.clone(), resource)]),
    )?;
    let mut streams = Vec::new();
    for i in 0..adapter.display().pages.len() {
        let batch = adapter
            .page(i, HintPolicy::Unhinted, MixedLimits::default())
            .map_err(|e| format!("batch: {e:?}"))?;
        streams.push(
            PdfCommandStream::from_mixed(&batch, StreamLimits::default())
                .map_err(|e| format!("stream: {e:?}"))?,
        );
    }
    let pdf = export(&streams, ExportLimits::default()).map_err(|e| format!("export: {e:?}"))?;
    std::fs::write(Path::new(&format!("{}.pdf", a[5])), pdf.bytes())?;
    std::fs::write(
        Path::new(&format!("{}.evidence.json", a[5])),
        pdf.evidence_bytes(),
    )?;
    println!(
        "pdf_sha256={} input_sha256={} registry_generation={}",
        digest(pdf.bytes()),
        digest(&bytes),
        registry.generation()
    );
    Ok(())
}
