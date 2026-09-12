//! Conservative dependency-aware incremental compilation.
//!
//! The parser always executes the complete finite language so macro definitions,
//! group restoration, diagnostics, and recovery retain exactly the clean-build
//! semantics. Reuse is limited to positioned per-block layout fragments, and is
//! allowed only when source mapping, the definitions actually read by the block,
//! preamble bytes, layout constraints, and entering flow state all match.
//!
//! This is not a general TeX incremental algorithm. Mutable category codes,
//! registers, assignments, conditionals, auxiliary files, output routines,
//! external effects, unsupported constructs, and malformed input force a full
//! layout rebuild. Any future construct is unsafe until its complete state and
//! side effects are represented in these cache checks. When in doubt, rebuild.

use crate::diagnostics::Diagnostic;
use crate::layout::{self, FlowState, LayoutCursor, Page, PlacedItem, TextItem};
use crate::math::{MathAtom, MathList, Nucleus};
use crate::parser::{self, Block, Inline, MacroDependency, SourceDocument};
use crate::Span;
use std::ops::Range;

pub use crate::layout::LayoutConstraints;

/// Per-revision evidence of how much block layout work was reused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReuseStats {
    pub blocks_total: usize,
    pub blocks_reused: usize,
    pub blocks_recomputed: usize,
    pub full_recompile: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompileOutput {
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncrementalResult {
    pub output: CompileOutput,
    pub stats: ReuseStats,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedBytes {
    pub old: Range<usize>,
    pub new: Range<usize>,
}

/// Smallest changed UTF-8-safe ranges in two snapshots.
pub fn changed_bytes(old: &str, new: &str) -> ChangedBytes {
    let old_bytes = old.as_bytes();
    let new_bytes = new.as_bytes();
    let shared = old_bytes.len().min(new_bytes.len());
    let mut prefix = 0;
    while prefix < shared && old_bytes[prefix] == new_bytes[prefix] {
        prefix += 1;
    }
    while prefix > 0 && (!old.is_char_boundary(prefix) || !new.is_char_boundary(prefix)) {
        prefix -= 1;
    }
    let mut suffix = 0;
    while suffix < old_bytes.len().saturating_sub(prefix)
        && suffix < new_bytes.len().saturating_sub(prefix)
        && old_bytes[old_bytes.len() - 1 - suffix] == new_bytes[new_bytes.len() - 1 - suffix]
    {
        suffix += 1;
    }
    while suffix > 0
        && (!old.is_char_boundary(old_bytes.len() - suffix)
            || !new.is_char_boundary(new_bytes.len() - suffix))
    {
        suffix -= 1;
    }
    ChangedBytes {
        old: prefix..old_bytes.len() - suffix,
        new: prefix..new_bytes.len() - suffix,
    }
}

#[derive(Debug, Clone)]
struct CachedBlock {
    block: Block,
    dependencies: Vec<MacroDependency>,
    prepared_state: FlowState,
    end_state: FlowState,
    placed: Vec<PlacedItem>,
}

#[derive(Debug, Clone)]
struct Revision {
    documents: Vec<(String, String)>,
    entry_path: String,
    constraints: LayoutConstraints,
    preamble_source: String,
    incremental_safe: bool,
    output: CompileOutput,
    blocks: Vec<CachedBlock>,
}

/// Previous revision and reusable block-layout fragments for one document.
#[derive(Debug, Default)]
pub struct Session {
    previous: Option<Revision>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(&mut self, text: &str, constraints: LayoutConstraints) -> IncrementalResult {
        self.compile_project(&[SourceDocument { path: "", text }], "", constraints)
    }

    /// Compile a complete supplied project while retaining reusable block layout.
    pub fn compile_project(
        &mut self,
        documents: &[SourceDocument<'_>],
        entry_path: &str,
        constraints: LayoutConstraints,
    ) -> IncrementalResult {
        let snapshot: Vec<(String, String)> = documents
            .iter()
            .map(|document| (document.path.to_string(), document.text.to_string()))
            .collect();
        if let Some(previous) = &self.previous {
            if previous.documents == snapshot
                && previous.entry_path == entry_path
                && previous.constraints == constraints
            {
                let total = previous.output.blocks.len();
                return IncrementalResult {
                    output: previous.output.clone(),
                    stats: ReuseStats {
                        blocks_total: total,
                        blocks_reused: total,
                        blocks_recomputed: 0,
                        full_recompile: false,
                    },
                };
            }
        }

        let parsed = parser::parse_project(documents, entry_path);
        let same_document_set = self.previous.as_ref().is_some_and(|previous| {
            previous.entry_path == entry_path
                && previous.documents.len() == snapshot.len()
                && previous
                    .documents
                    .iter()
                    .zip(&snapshot)
                    .all(|((old_path, _), (new_path, _))| old_path == new_path)
        });
        let can_reuse = self.previous.as_ref().is_some_and(|previous| {
            same_document_set
                && previous.incremental_safe
                && parsed.incremental_safe
                && previous.constraints == constraints
                && previous.preamble_source == parsed.preamble_source
        });
        let changes: Vec<ChangedBytes> = self.previous.as_ref().map_or_else(Vec::new, |previous| {
            previous
                .documents
                .iter()
                .zip(&snapshot)
                .map(|((_, old), (_, new))| changed_bytes(old, new))
                .collect()
        });
        let deltas: Vec<isize> = self.previous.as_ref().map_or_else(Vec::new, |previous| {
            previous
                .documents
                .iter()
                .zip(&snapshot)
                .map(|((_, old), (_, new))| new.len() as isize - old.len() as isize)
                .collect()
        });

        let mut cursor = LayoutCursor::new(constraints);
        let mut cache = Vec::with_capacity(parsed.blocks.len());
        let mut stats = ReuseStats {
            blocks_total: parsed.blocks.len(),
            full_recompile: !can_reuse,
            ..ReuseStats::default()
        };

        for (index, block) in parsed.blocks.iter().enumerate() {
            let dependencies = parsed.block_dependencies[index].clone();
            let prepared_state = cursor.prepare_block(block);
            let candidate = if can_reuse {
                self.previous.as_ref().and_then(|previous| {
                    previous.blocks.iter().find(|cached| {
                        cached.dependencies == dependencies
                            && shift_block(&cached.block, &changes, &deltas).as_ref() == Some(block)
                    })
                })
            } else {
                None
            };

            let placed = if let Some(cached) = candidate {
                if prepared_state.same_geometry(cached.prepared_state) {
                    let shifted = shift_placed(&cached.placed, &changes, &deltas)
                        .expect("candidate spans were already validated");
                    cursor.append_reused(&shifted, cached.end_state);
                    stats.blocks_reused += 1;
                    shifted
                } else {
                    stats.blocks_recomputed += 1;
                    cursor.render_prepared_block(block)
                }
            } else {
                stats.blocks_recomputed += 1;
                cursor.render_prepared_block(block)
            };
            let end_state = cursor.state();
            cache.push(CachedBlock {
                block: block.clone(),
                dependencies,
                prepared_state,
                end_state,
                placed,
            });
        }

        let output = CompileOutput {
            blocks: parsed.blocks,
            diagnostics: parsed.diagnostics,
            pages: cursor.into_pages(),
        };
        self.previous = Some(Revision {
            documents: snapshot,
            entry_path: entry_path.to_string(),
            constraints,
            preamble_source: parsed.preamble_source,
            incremental_safe: parsed.incremental_safe,
            output: output.clone(),
            blocks: cache,
        });
        IncrementalResult { output, stats }
    }
}

/// Authoritative clean compile for equivalence checks and callers without a session.
pub fn compile_full(text: &str, constraints: LayoutConstraints) -> CompileOutput {
    compile_full_project(&[SourceDocument { path: "", text }], "", constraints)
}

/// Authoritative clean compile for a complete supplied project.
pub fn compile_full_project(
    documents: &[SourceDocument<'_>],
    entry_path: &str,
    constraints: LayoutConstraints,
) -> CompileOutput {
    let parsed = parser::parse_project(documents, entry_path);
    let pages = layout::layout_with_constraints(&parsed.blocks, constraints);
    CompileOutput {
        blocks: parsed.blocks,
        diagnostics: parsed.diagnostics,
        pages,
    }
}

fn mapped_span(span: Span, changes: &[ChangedBytes], deltas: &[isize]) -> Option<Span> {
    let change = changes.get(span.document.0)?;
    let delta = *deltas.get(span.document.0)?;
    if span.end <= change.old.start {
        Some(span)
    } else if span.start >= change.old.end {
        Some(shift_span(span, delta))
    } else {
        None
    }
}

fn shift_span(span: Span, delta: isize) -> Span {
    Span::in_document(
        span.document,
        span.start
            .checked_add_signed(delta)
            .expect("valid span shift"),
        span.end
            .checked_add_signed(delta)
            .expect("valid span shift"),
    )
}

fn shift_block(block: &Block, changes: &[ChangedBytes], deltas: &[isize]) -> Option<Block> {
    Some(match block {
        Block::Paragraph(inlines) => Block::Paragraph(shift_inlines(inlines, changes, deltas)?),
        Block::Heading { level, content } => Block::Heading {
            level: *level,
            content: shift_inlines(content, changes, deltas)?,
        },
    })
}

fn shift_inlines(
    inlines: &[Inline],
    changes: &[ChangedBytes],
    deltas: &[isize],
) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::Text { text, span } => Some(Inline::Text {
                text: text.clone(),
                span: mapped_span(*span, changes, deltas)?,
            }),
            Inline::LineBreak { span } => Some(Inline::LineBreak {
                span: mapped_span(*span, changes, deltas)?,
            }),
            Inline::Math {
                list,
                display,
                span,
            } => Some(Inline::Math {
                list: shift_math_list(list, changes, deltas)?,
                display: *display,
                span: mapped_span(*span, changes, deltas)?,
            }),
        })
        .collect()
}

fn shift_math_list(
    list: &MathList,
    changes: &[ChangedBytes],
    deltas: &[isize],
) -> Option<MathList> {
    Some(MathList {
        atoms: list
            .atoms
            .iter()
            .map(|atom| {
                Some(MathAtom {
                    nucleus: match &atom.nucleus {
                        Nucleus::Symbol(text) => Nucleus::Symbol(text.clone()),
                        Nucleus::Fraction {
                            numerator,
                            denominator,
                        } => Nucleus::Fraction {
                            numerator: shift_math_list(numerator, changes, deltas)?,
                            denominator: shift_math_list(denominator, changes, deltas)?,
                        },
                        Nucleus::Radical(list) => {
                            Nucleus::Radical(shift_math_list(list, changes, deltas)?)
                        }
                    },
                    span: mapped_span(atom.span, changes, deltas)?,
                    // An absent script stays absent; a present one that cannot be
                    // shifted fails the whole mapping, so the caller falls back to a
                    // full recompile rather than emitting a stale span.
                    superscript: match atom.superscript.as_ref() {
                        Some(list) => Some(shift_math_list(list, changes, deltas)?),
                        None => None,
                    },
                    subscript: match atom.subscript.as_ref() {
                        Some(list) => Some(shift_math_list(list, changes, deltas)?),
                        None => None,
                    },
                })
            })
            .collect::<Option<Vec<_>>>()?,
    })
}

fn shift_placed(
    items: &[PlacedItem],
    changes: &[ChangedBytes],
    deltas: &[isize],
) -> Option<Vec<PlacedItem>> {
    items
        .iter()
        .map(|placed| {
            Some(PlacedItem {
                page_index: placed.page_index,
                item: TextItem {
                    text: placed.item.text.clone(),
                    x_pt: placed.item.x_pt,
                    baseline_y_pt: placed.item.baseline_y_pt,
                    font_size_pt: placed.item.font_size_pt,
                    span: mapped_span(placed.item.span, changes, deltas)?,
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_byte_identical_to_full(
        result: &IncrementalResult,
        text: &str,
        constraints: LayoutConstraints,
    ) {
        let full = compile_full(text, constraints);
        let incremental_bytes = format!("{:#?}", result.output).into_bytes();
        let full_bytes = format!("{full:#?}").into_bytes();
        assert_eq!(incremental_bytes, full_bytes);
    }

    fn compile_edit(old: &str, new: &str) -> IncrementalResult {
        let constraints = LayoutConstraints::default();
        let mut session = Session::new();
        let cold = session.compile(old, constraints);
        assert_byte_identical_to_full(&cold, old, constraints);
        let result = session.compile(new, constraints);
        eprintln!("ReuseStats: {:?}", result.stats);
        assert_byte_identical_to_full(&result, new, constraints);
        result
    }

    #[test]
    fn edit_inside_one_paragraph_matches_clean_build() {
        let result = compile_edit(
            "Alpha beta.\n\nMiddle words here.\n\nOmega final.",
            "Alpha beta.\n\nMiddle changed here.\n\nOmega final.",
        );
        assert_eq!(result.stats.blocks_total, 3);
        assert!(result.stats.blocks_reused >= 2);
    }

    #[test]
    fn heading_and_math_use_the_same_cursor_as_clean_layout() {
        let result = compile_edit(
            "\\section{Measured $x^2$ heading}\n\nBody with $\\frac{1}{2}$.\n\nTail.",
            "\\section{Measured $x^2$ heading}\n\nEdited body with $\\frac{1}{2}$.\n\nTail.",
        );
        assert!(result.stats.blocks_reused >= 1);
    }

    #[test]
    fn edits_at_start_and_end_match_clean_build() {
        let constraints = LayoutConstraints::default();
        let original = "First paragraph.\n\nSecond paragraph.\n\nLast paragraph.";
        let at_start = "New first paragraph.\n\nSecond paragraph.\n\nLast paragraph.";
        let at_end = "New first paragraph.\n\nSecond paragraph.\n\nLast paragraph changed.";
        let mut session = Session::new();
        session.compile(original, constraints);
        let start_result = session.compile(at_start, constraints);
        eprintln!("start ReuseStats: {:?}", start_result.stats);
        assert_byte_identical_to_full(&start_result, at_start, constraints);
        let end_result = session.compile(at_end, constraints);
        eprintln!("end ReuseStats: {:?}", end_result.stats);
        assert_byte_identical_to_full(&end_result, at_end, constraints);
    }

    #[test]
    fn edit_spanning_paragraph_boundary_matches_clean_build() {
        let result = compile_edit(
            "One paragraph.\n\nTwo paragraph.\n\nThree paragraph.",
            "One paragraph joined to Two paragraph.\n\nThree paragraph.",
        );
        assert_eq!(result.stats.blocks_total, 2);
    }

    #[test]
    fn multibyte_insertion_shifts_later_spans_by_byte_delta() {
        let new = "A café closes.\n\nLater paragraph stays.";
        let result = compile_edit("A cafe closes.\n\nLater paragraph stays.", new);
        let later = result
            .output
            .pages
            .iter()
            .flat_map(|page| &page.items)
            .find(|item| item.text == "Later")
            .expect("later item");
        assert_eq!(&new[later.span.start..later.span.end], "Later");
        assert!(result.stats.blocks_reused >= 1);
    }

    #[test]
    fn changed_line_count_repositions_later_blocks() {
        let short = "short opening.\n\nLater paragraph.";
        let long_words = "wide ".repeat(120);
        let long = format!("{}\n\nLater paragraph.", long_words);
        let result = compile_edit(short, &long);
        assert_eq!(result.stats.blocks_reused, 0);
        let later = result
            .output
            .pages
            .iter()
            .flat_map(|page| &page.items)
            .find(|item| item.text == "Later")
            .expect("later item");
        assert!(later.baseline_y_pt > 100.0);
    }

    #[test]
    fn redefining_macro_invalidates_untouched_reader() {
        let old = "\\newcommand{\\term}{base}\\renewcommand{\\term}{old}\n\nPlain before.\n\nUse \\term here.";
        let new = "\\newcommand{\\term}{base}\\renewcommand{\\term}{new}\n\nPlain before.\n\nUse \\term here.";
        let result = compile_edit(old, new);
        assert!(result
            .output
            .pages
            .iter()
            .flat_map(|page| &page.items)
            .any(|item| item.text == "new"));
        assert_eq!(result.stats.blocks_reused, 1);
        assert_eq!(result.stats.blocks_recomputed, 1);
    }

    #[test]
    fn scoped_macro_change_does_not_invalidate_global_reader() {
        let old = "\\newcommand{\\term}{global}{\\renewcommand{\\term}{local}Scoped \\term.}\n\nGlobal \\term.";
        let new = "\\newcommand{\\term}{global}{\\renewcommand{\\term}{inner}Scoped \\term.}\n\nGlobal \\term.";
        let result = compile_edit(old, new);
        assert_eq!(result.stats.blocks_total, 2);
        assert_eq!(result.stats.blocks_reused, 1);
        assert_eq!(result.stats.blocks_recomputed, 1);
    }

    #[test]
    fn adding_usepackage_forces_full_recompile() {
        let old = "\\documentclass{article}\n\\begin{document}\nOne.\n\nTwo.\n\\end{document}";
        let new = "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\nOne.\n\nTwo.\n\\end{document}";
        let result = compile_edit(old, new);
        assert!(result.stats.full_recompile);
        assert_eq!(result.stats.blocks_reused, 0);
        assert_eq!(result.stats.blocks_recomputed, 2);
    }

    #[test]
    fn malformed_edit_has_partial_output_and_source_diagnostics() {
        let new = "Good text {unclosed and \\unknown here.\n\nLater text.";
        let result = compile_edit("Good text.\n\nLater text.", new);
        assert!(result.stats.full_recompile);
        assert!(!result.output.pages[0].items.is_empty());
        assert!(result.output.diagnostics.iter().all(|diag| {
            diag.span.is_some()
                && diag
                    .recovery
                    .as_deref()
                    .is_some_and(|text| !text.is_empty())
        }));
    }

    #[test]
    fn layout_constraint_change_forces_full_recompile() {
        let text = "One paragraph.\n\nTwo paragraph.";
        let mut session = Session::new();
        session.compile(text, LayoutConstraints::default());
        let constraints = LayoutConstraints {
            font_size_pt: 13.0,
            measure_pt: 320.0,
        };
        let result = session.compile(text, constraints);
        eprintln!("constraint ReuseStats: {:?}", result.stats);
        assert!(result.stats.full_recompile);
        assert_byte_identical_to_full(&result, text, constraints);
    }
}
