//! Bounded synthetic measurement: 24 documents, 80 lines each, 12 edits.
//! Times are local samples; exact clean-rebuild equality is checked at each step.
use flashtex_project_index::ProjectIndex;
use std::time::Instant;

fn main() {
    let mut sources: Vec<(String, u64, String)> = (0..24)
        .map(|file| {
            let text = if file == 0 {
                definitions(false)
            } else {
                (0..80)
                    .map(|line| {
                        format!("東京 paragraph {file}:{line} \\ref{{shared{}}}\n", file % 2)
                    })
                    .collect()
            };
            (format!("part-{file:02}.tex"), 0, text)
        })
        .collect();
    let total_input_bytes: usize = sources.iter().map(|(_, _, text)| text.len()).sum();
    let mut index = ProjectIndex::new("measured").unwrap();
    let cold_start = Instant::now();
    for (file, revision, text) in &sources {
        index.replace_document(file, *revision, text).unwrap();
    }
    let cold_nanos = cold_start.elapsed().as_nanos();
    let mut edit_times = Vec::new();
    let mut clean_times = Vec::new();
    let mut reindexed_bytes = 0;
    let mut reused = 0;
    let mut rechecked = 0;
    for revision in 1..=12 {
        sources[0].1 = revision;
        sources[0].2 = definitions(revision % 2 == 1);
        let update = index
            .replace_document(&sources[0].0, revision, &sources[0].2)
            .unwrap();
        edit_times.push(update.metrics.total_elapsed_nanos);
        reindexed_bytes += update.metrics.input_bytes_reindexed;
        reused += update.metrics.document_indexes_reused;
        rechecked += update.metrics.reference_documents_rechecked;
        let mut clean = ProjectIndex::new("clean").unwrap();
        let clean_start = Instant::now();
        for (file, rev, text) in &sources {
            clean.replace_document(file, *rev, text).unwrap();
        }
        clean_times.push(clean_start.elapsed().as_nanos());
        assert_eq!(
            index.symbols(&index.snapshot()).unwrap(),
            clean.symbols(&clean.snapshot()).unwrap()
        );
        assert_eq!(
            index.unresolved_references(&index.snapshot()).unwrap(),
            clean.unresolved_references(&clean.snapshot()).unwrap()
        );
    }
    edit_times.sort_unstable();
    clean_times.sort_unstable();
    println!("{{\"workload\":\"24 documents, 80 lines each, 12 definition edits\",\"synthetic\":true,\"clean_equivalence_comparisons\":12,\"all_equivalent\":true,\"initial_project_bytes\":{total_input_bytes},\"cold_nanos\":{cold_nanos},\"edit_median_nanos\":{},\"edit_min_nanos\":{},\"edit_max_nanos\":{},\"clean_median_nanos\":{},\"total_edit_input_bytes_reindexed\":{reindexed_bytes},\"total_document_indexes_reused\":{reused},\"total_reference_documents_rechecked\":{rechecked}}}",
        edit_times[edit_times.len()/2], edit_times[0], edit_times[edit_times.len()-1], clean_times[clean_times.len()/2]);
}

fn definitions(alternate: bool) -> String {
    let first = if alternate { "alternate0" } else { "shared0" };
    let mut text = format!("\\label{{{first}}} \\label{{shared1}}\n");
    for line in 0..80 {
        text.push_str(&format!("Ordinary definition paragraph {line}.\n"));
    }
    text
}
