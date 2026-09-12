//! Opt-in actual pipeline display -> immutable registry CFF -> exact PDF.
//! Args: DISPLAY REQUEST FONT_DIRECTORY LICENSE OUTPUT_PREFIX. No source rewriting.
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
    if a.len() != 6 && !(a.len() == 7 && a[6] == "--searchable") {
        return Err("DISPLAY REQUEST FONT_DIRECTORY LICENSE OUTPUT_PREFIX [--searchable]".into());
    }
    let bytes = std::fs::read(&a[1])?;
    let Message::DisplayList(list) = parse(&bytes)?.message else {
        return Err("display required".into());
    };
    if list.fonts.is_empty() || list.fonts.len() > 64 {
        return Err("probe font count limit".into());
    }
    let license = std::fs::read(&a[4])?;
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("LICENSE"), &license)?;
    let mut loaded = BTreeMap::new();
    let mut total = 0u64;
    for (entry_index, file) in std::fs::read_dir(&a[3])?.take(257).enumerate() {
        let file = file?;
        if entry_index >= 256 {
            return Err("font directory entry limit".into());
        }
        if file.path().extension().and_then(|s| s.to_str()) != Some("otf") {
            continue;
        }
        let size = file.metadata()?.len();
        total = total.checked_add(size).ok_or("font byte limit")?;
        if total > 64 * 1024 * 1024 {
            return Err("font byte limit".into());
        }
        let bytes = std::fs::read(file.path())?;
        loaded.insert(digest(&bytes), bytes);
    }
    let mut entries = Vec::new();
    for (index, f) in list.fonts.iter().enumerate() {
        let font = loaded
            .get(&f.sha256)
            .ok_or("missing exact declared raw font digest")?;
        let path = format!("font-{index}.otf");
        std::fs::write(temp.path().join(&path), font)?;
        entries.push(RegistryEntry {
            binding: StyleBinding {
                family: f.font_id.clone(),
                weight: 400,
                style: FontStyle::Upright,
            },
            resource: ManifestEntry {
                font: FontDescriptor {
                    font_id: f.font_id.clone(),
                    sha256: digest(font),
                    byte_length: f.byte_length,
                    format: "static-cff".into(),
                    face_index: f.face_index,
                    units_per_em: f.units_per_em,
                    glyph_count: f.glyph_count,
                    postscript_name: f.postscript_name.clone(),
                },
                path,
                license: LicenseMetadata {
                    identifier: "LicenseRef-Explicit".into(),
                    copyright: "See supplied license".into(),
                    source: a[4].clone(),
                    text_path: "LICENSE".into(),
                    text_sha256: digest(&license),
                    embedding_permission: EmbeddingPermission::Unknown,
                },
            },
        });
    }
    let manifest = RegistryManifest {
        schema_version: 1,
        entries,
    };
    std::fs::write(
        temp.path().join("fonts.json"),
        serde_json::to_vec(&manifest)?,
    )?;
    let root = ProjectRoot::open(temp.path())?;
    let registry = ProjectFontRegistry::load(&root, "fonts.json", RegistryLimits::default())?;
    let mut resources = BTreeMap::new();
    for entry in &manifest.entries {
        let RegistryResource::Cff(resource) = registry.resource(&entry.binding)? else {
            return Err("CFF required".into());
        };
        resources.insert(entry.resource.font.font_id.clone(), resource);
    }
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
    let adapter = PipelineCff::bind(&bytes, &caps, &docs, &resources)?;
    if a.len() == 7 {
        let pdf = adapter.export_searchable(64 * 1024 * 1024)?;
        std::fs::write(format!("{}.pdf", a[5]), &pdf.bytes)?;
        let evidence = serde_json::json!({"format":"flashtex-verified-searchable-pipeline-probe-v1","input_sha256":pdf.input_sha256,"pdf_sha256":digest(&pdf.bytes),"fonts":adapter.display().fonts,"license_sha256":digest(&license),"documents":adapter.display().documents,"glyphs":pdf.report.glyphs,"pages":pdf.report.pages,"visual_parity":null,"source_text_policy":"producer-cluster-text; source spans retained separately","registry_generation":registry.generation()});
        std::fs::write(
            format!("{}.evidence.json", a[5]),
            serde_json::to_vec_pretty(&evidence)?,
        )?;
        println!(
            "searchable_pdf_sha256={} input_sha256={}",
            digest(&pdf.bytes),
            pdf.input_sha256
        );
        return Ok(());
    }
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
