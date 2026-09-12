//! Rev 3: adversarial bounds. Attacks the rev-2 stabilizer (`converge`,
//! `entry`, `leader`, `stabilize`) with hostile models and inputs. Every
//! case here must resolve to a typed error or a bounded, deterministic
//! result — never a panic, a hang, or an unbounded loop.

use flashtex_toc_layout::*;

// ---------------------------------------------------------------------
// Long-period oscillation: the rev-2 cycle detector classifies
// non-convergence by scanning the *completed* history for the earliest
// repeated candidate. That only proves a cycle when the repeat actually
// falls inside the `MAX_CONVERGENCE_ITERATIONS + 1`-element history — so
// a period exactly at the bound is still caught, but a period one longer
// than the bound is indistinguishable, from the outside, from a model
// that never repeats at all, and is reported as `Unresolved`, not `Cycle`.
// ---------------------------------------------------------------------

/// Walks a fixed cycle of distinct candidates in order, wrapping around.
/// Deterministic and pure in `candidate` (as `FrontMatterModel` requires):
/// a candidate not in the cycle is treated as the start of it, so the walk
/// is well-defined regardless of `initial_guess`.
struct LongPeriodOscillatingModel {
    cycle: Vec<u32>,
}

impl FrontMatterModel for LongPeriodOscillatingModel {
    fn pages_for(&self, candidate: u32) -> u32 {
        let idx = self.cycle.iter().position(|&v| v == candidate).unwrap_or(0);
        self.cycle[(idx + 1) % self.cycle.len()]
    }
}

#[test]
fn long_period_oscillation_shorter_than_bound_is_classified_as_a_cycle() {
    // Period 5, well inside MAX_CONVERGENCE_ITERATIONS (8): the 9-candidate
    // history sees the full period plus its repeat.
    let model = LongPeriodOscillatingModel {
        cycle: vec![10, 11, 12, 13, 14],
    };
    match converge_front_matter_pages(&model, 10) {
        Err(ConvergenceError::Cycle {
            bound,
            history,
            cycle_start,
            cycle,
        }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            assert_eq!(history, vec![10, 11, 12, 13, 14, 10, 11, 12, 13]);
            assert_eq!(cycle_start, 0);
            assert_eq!(cycle, vec![10, 11, 12, 13, 14]);
        }
        other => panic!("expected Cycle, got {other:?}"),
    }
}

#[test]
fn long_period_oscillation_exactly_at_the_bound_is_still_classified_as_a_cycle() {
    // Period 8, exactly MAX_CONVERGENCE_ITERATIONS: the wraparound lands
    // precisely on the last history slot, so it is still detected.
    let model = LongPeriodOscillatingModel {
        cycle: vec![100, 101, 102, 103, 104, 105, 106, 107],
    };
    match converge_front_matter_pages(&model, 100) {
        Err(ConvergenceError::Cycle {
            bound,
            history,
            cycle_start,
            cycle,
        }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            assert_eq!(history, vec![100, 101, 102, 103, 104, 105, 106, 107, 100]);
            assert_eq!(cycle_start, 0);
            assert_eq!(cycle, vec![100, 101, 102, 103, 104, 105, 106, 107]);
        }
        other => panic!("expected Cycle, got {other:?}"),
    }
}

#[test]
fn long_period_oscillation_one_longer_than_the_bound_is_unresolved_not_a_cycle() {
    // Period 9 — one longer than MAX_CONVERGENCE_ITERATIONS (8). The model
    // truly does cycle forever, but the bounded 9-candidate history never
    // observes the wraparound (it would need a 10th candidate to see
    // history[9] == history[0]), so every candidate tried within the bound
    // is distinct. This is the documented, deliberate cost of a fixed
    // observation window: a real cycle whose period exceeds the bound is
    // reported as `Unresolved`, exactly like a genuinely divergent model,
    // never mislabeled as a proven `Cycle` on stale evidence.
    let model = LongPeriodOscillatingModel {
        cycle: vec![200, 201, 202, 203, 204, 205, 206, 207, 208],
    };
    match converge_front_matter_pages(&model, 200) {
        Err(ConvergenceError::Unresolved { bound, history }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            assert_eq!(history, vec![200, 201, 202, 203, 204, 205, 206, 207, 208]);
            let mut sorted = history.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), history.len(), "every candidate was distinct");
        }
        other => panic!("expected Unresolved, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// A model that increases without bound. Uses doubling (steeper than the
// existing +1 `EverIncreasingModel` in tests/toc.rs) to prove the bound
// holds regardless of how fast the model runs away, and never panics on
// overflow.
// ---------------------------------------------------------------------

struct DoublingModel;
impl FrontMatterModel for DoublingModel {
    fn pages_for(&self, candidate: u32) -> u32 {
        candidate.saturating_mul(2).saturating_add(1)
    }
}

#[test]
fn ever_growing_model_is_bounded_and_never_panics() {
    let result = converge_front_matter_pages(&DoublingModel, 1);
    match result {
        Err(ConvergenceError::Unresolved { bound, history }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            assert_eq!(history, vec![1, 3, 7, 15, 31, 63, 127, 255, 511]);
        }
        other => panic!("expected Unresolved, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// A model that returns wildly varying values (an LCG-style hash of the
// candidate). No claim about which variant it lands on — the point is
// that the solver never panics on the arithmetic and always terminates
// within the bound with a typed result.
// ---------------------------------------------------------------------

struct WildlyVaryingModel;
impl FrontMatterModel for WildlyVaryingModel {
    fn pages_for(&self, candidate: u32) -> u32 {
        candidate.wrapping_mul(1_103_515_245).wrapping_add(12_345)
    }
}

#[test]
fn wildly_varying_model_terminates_with_a_bounded_typed_result() {
    let result = converge_front_matter_pages(&WildlyVaryingModel, 0xDEAD_BEEF);
    match result {
        Ok(_) => {}
        Err(ConvergenceError::Cycle { bound, history, .. })
        | Err(ConvergenceError::Unresolved { bound, history }) => {
            assert_eq!(bound, MAX_CONVERGENCE_ITERATIONS);
            assert!(history.len() <= MAX_CONVERGENCE_ITERATIONS + 1);
        }
    }
}

// ---------------------------------------------------------------------
// Entries numbering in the tens of thousands: stabilize_toc must handle
// this without a panic, a hang, or quadratic blowup.
// ---------------------------------------------------------------------

struct FixedNeedModel {
    needed: u32,
}
impl FrontMatterModel for FixedNeedModel {
    fn pages_for(&self, _candidate: u32) -> u32 {
        self.needed
    }
}

#[test]
fn tens_of_thousands_of_entries_stabilize_without_panic_or_hang() {
    const COUNT: u32 = 30_000;
    let model = FixedNeedModel { needed: 5 };
    let entries: Vec<SourcedEntry> = (0..COUNT)
        .map(|i| SourcedEntry {
            source: SourceId::new("massive.tex", "rev-1"),
            sequence: i,
            entry: RelativeEntry::new(format!("Entry {i}"), 0, i + 1).unwrap(),
        })
        .collect();

    let toc = stabilize_toc(&entries, &model, 0).unwrap();
    assert_eq!(toc.front_matter_pages, 5);
    assert_eq!(toc.entries.len(), COUNT as usize);
    assert_eq!(toc.entries[0].record.page, 6); // 5 + 1
    assert_eq!(
        toc.entries[(COUNT - 1) as usize].record.page,
        5 + COUNT // 5 + i+1 for the last i = COUNT - 1
    );
}

// ---------------------------------------------------------------------
// A page number at integer maximum: accepted when it does not overflow,
// a typed `PageOverflow` (never a panic) when it would.
// ---------------------------------------------------------------------

#[test]
fn page_number_at_exact_integer_maximum_is_accepted() {
    let record = EntryRecord::new("Last Page", u32::MAX, 0).unwrap();
    assert_eq!(record.page, u32::MAX);

    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(50.0, 0.0).unwrap();
    let laid_out = layout_entry(&record, line, &measure).unwrap();
    assert_eq!(laid_out.page_label, "4294967295");
    assert_eq!(laid_out.page_label_width, 10.0);
    // indent(0) + title(9, "Last Page") + leaders + gap + label(10) == 50.
    assert_eq!(laid_out.leader_count, 31); // available = 50 - 9 - 10 = 31
    assert_eq!(laid_out.leader_width, 31.0);
    assert_eq!(laid_out.gap_before_page, 0.0);
}

#[test]
fn relative_entry_resolving_past_integer_maximum_is_a_typed_overflow_not_a_panic() {
    let rel = RelativeEntry::new("Overflows", 0, 1).unwrap();
    assert_eq!(rel.resolve(u32::MAX), Err(EntryError::PageOverflow));
}

#[test]
fn relative_entry_body_offset_at_exact_maximum_resolves_cleanly_from_zero() {
    let rel = RelativeEntry::new("Exactly Max", 0, u32::MAX).unwrap();
    assert_eq!(rel.resolve(0).unwrap().page, u32::MAX);
}

// ---------------------------------------------------------------------
// A title of enormous length: must not panic while measuring or summing
// widths, and must resolve to a typed Overflow when it cannot fit.
// ---------------------------------------------------------------------

#[test]
fn enormous_title_does_not_panic_and_overflows_typed_when_it_cannot_fit() {
    let huge_title = "x".repeat(1_000_000);
    // Constructing the entry itself never inspects length beyond emptiness.
    let entry = EntryRecord::new(huge_title.clone(), 1, 0).unwrap();

    let measure = CharWidthMeasure::new(1.0, 1.0);
    let line = LineBox::new(100.0, 0.0).unwrap();
    match layout_entry(&entry, line, &measure) {
        Err(LayoutError::Overflow { title, deficit }) => {
            assert_eq!(title, huge_title);
            assert!(deficit > 0.0);
        }
        other => panic!("expected Overflow, got {other:?}"),
    }
}

#[test]
fn enormous_title_that_fits_a_correspondingly_enormous_line_measures_exactly() {
    let huge_title = "y".repeat(500_000);
    let entry = EntryRecord::new(huge_title, 1, 0).unwrap();
    let measure = CharWidthMeasure::new(1.0, 1.0);
    // Wide enough line: 500_000 (title) + 1 (page "1") + slack for leaders.
    let line = LineBox::new(500_010.0, 0.0).unwrap();
    let laid_out = layout_entry(&entry, line, &measure).unwrap();
    assert_eq!(laid_out.title_width, 500_000.0);
    assert_eq!(laid_out.page_label_width, 1.0);
}

// ---------------------------------------------------------------------
// Zero or negative line width: a typed LineBoxError, never a panic,
// with the exact offending values preserved for the caller.
// ---------------------------------------------------------------------

#[test]
fn zero_line_width_is_a_typed_error_with_exact_offending_values() {
    assert_eq!(
        LineBox::new(0.0, 2.0),
        Err(LineBoxError {
            width: 0.0,
            indent_unit: 2.0
        })
    );
}

#[test]
fn negative_line_width_is_a_typed_error_with_exact_offending_values() {
    assert_eq!(
        LineBox::new(-42.5, 0.0),
        Err(LineBoxError {
            width: -42.5,
            indent_unit: 0.0
        })
    );
}

// ---------------------------------------------------------------------
// Duplicate sequence numbers: stabilize_toc has no `sequence`-uniqueness
// contract (documented in coordination/daniel-contents.md as the caller's
// responsibility). Duplicates must still produce a bounded, deterministic
// result — never a panic — and ties are broken by input order, since
// `sort_by_key` is a stable sort.
// ---------------------------------------------------------------------

#[test]
fn duplicate_sequence_numbers_do_not_panic_and_tie_break_by_input_order() {
    let model = FixedNeedModel { needed: 1 };
    let entries = vec![
        SourcedEntry {
            source: SourceId::new("doc.tex", "rev-A"),
            sequence: 0,
            entry: RelativeEntry::new("A", 0, 1).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("doc.tex", "rev-A"),
            sequence: 0,
            entry: RelativeEntry::new("B", 0, 2).unwrap(),
        },
        SourcedEntry {
            source: SourceId::new("doc.tex", "rev-A"),
            sequence: 0,
            entry: RelativeEntry::new("C", 0, 3).unwrap(),
        },
    ];

    let toc = stabilize_toc(&entries, &model, 0).unwrap();
    assert_eq!(toc.entries.len(), 3);
    let titles: Vec<&str> = toc
        .entries
        .iter()
        .map(|e| e.record.title.as_str())
        .collect();
    assert_eq!(titles, vec!["A", "B", "C"]);
    let pages: Vec<u32> = toc.entries.iter().map(|e| e.record.page).collect();
    assert_eq!(pages, vec![2, 3, 4]);
}
