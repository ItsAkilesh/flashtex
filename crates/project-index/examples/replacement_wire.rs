//! Standalone deterministic wire example: prints a proposal; does not apply it.
use flashtex_project_index::{ProjectIndex, SearchRequest, MAX_REPLACEMENT_WIRE_BYTES};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = ProjectIndex::new("native-example")?;
    index.replace_document("chapters/α.tex", u64::MAX, "% α\nα")?;
    let search =
        index.search_literal(&index.snapshot(), &SearchRequest::literal("α"), || false)?;
    let proposal = index.plan_literal_replacement(&search, "β\n\"\\\u{0000}")?;
    println!(
        "{}",
        index.serialize_literal_replacement_plan(&proposal, MAX_REPLACEMENT_WIRE_BYTES)?
    );
    Ok(())
}
