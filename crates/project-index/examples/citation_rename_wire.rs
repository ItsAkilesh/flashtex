//! Prints a validated proposal only; no source mutation or native application.
use flashtex_project_index::{ProjectIndex, MAX_REPLACEMENT_WIRE_BYTES};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = ProjectIndex::new("citation-native-example")?;
    index.replace_bibliography_document("refs.bib", u64::MAX, "@book{κ,title={κ}}")?;
    index.replace_document("a.tex", 1, r"\cite[see]{κ,other}")?;
    let plan = index.plan_citation_rename(&index.snapshot(), "κ", "引用:2026")?;
    println!(
        "{}",
        index.serialize_citation_rename_plan(&plan, MAX_REPLACEMENT_WIRE_BYTES)?
    );
    Ok(())
}
