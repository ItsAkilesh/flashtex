//! `tabular`/`tabular*` in the pipeline: the LaTeX kernel's alignment model
//! and booktabs' rules, laid out over the compiler's parsed table
//! (`flashtex_compiler::tabular::Tabular`, whose column templates are built
//! the way `\@mkpream` builds the `\halign` preamble) with cells shaped by
//! the pipeline's own TFM text path (`typeset::Context::table_box`).
//!
//! Citations are to TeX Live 2026 `latex.ltx` and booktabs v1.61803398.
//!
//! * `\@tabular` (latex.ltx 16560): `\leavevmode\hbox{$ ... $}` with
//!   `\m@th`; `\@array` (16564) opens `\vtop` for `[t]`, `\vbox` for `[b]`,
//!   else `\vcenter` (centred on the math axis of the current size: cmsy's
//!   `axis_height` is 0.25 em), sets `\lineskip\z@skip\baselineskip\z@skip`
//!   so rows abut, and starts every row with `\@arstrut`, a rule of height
//!   `\arraystretch\ht\strutbox` and depth `\arraystretch\dp\strutbox`
//!   (`\strutbox` is `.7\baselineskip`/`.3\baselineskip` of the size in
//!   force, size1x.clo `\@setfontsize`).
//! * `\@tabclassz` (16652): `l`/`c`/`r` are `\hskip1sp\ignorespaces #\unskip`
//!   with `\hfil` on the open sides; `\@tabacol` (16629) puts `\tabcolsep`
//!   before and after each; `\@arrayrule` (16716) is
//!   `\hskip-.5\arrayrulewidth\vrule\hskip-.5\arrayrulewidth`, so a `|` takes
//!   no width and is centred on its boundary, running the full row height;
//!   `\@classi` puts `\doublerulesep` between `||`, `\@classii` half a rule
//!   width before `@` after `|`.
//! * `\halign` (TeX §801): a column is as wide as its widest entry; a
//!   spanned entry (`\multicolumn`, 16603) pushes only its excess over the
//!   columns before its last one into that last column; a column with no
//!   entry is zero-wide with zero `\tabskip` after it. `tabular*` (16557) is
//!   `\halign to<width>`: the leftover goes to `\extracolsep{\fill}`
//!   `\tabskip` glue; the glue after the last column is `\z@skip`.
//! * `p{w}` (`\@classv` 16702, `\@startpbox`/`\@endpbox` 16755): a `\vtop`
//!   of `\hsize` w under `\@arrayparboxrestore` (16272: no indent,
//!   `\parfillskip\@flushglue`, `\normalbaselineskip`, `\sloppy`) whose last
//!   line gets `\@finalstrut\@arstrutbox` (16396: the strut's depth).
//! * `\\[d]` (`\@argtabularcr` 16593): `d > 0` deepens the row to
//!   `d + \dp\@arstrutbox`; otherwise `\cr\noalign{\vskip d}`.
//! * `\hline` (16728): a full-width `\arrayrulewidth` rule taking its height;
//!   a second `\hline` directly after it is `\doublerulesep` below its top.
//!   `\cline{a-b}` (16738): an `\omit`ted row of leaders `\arrayrulewidth`
//!   high from column a's left edge to column b's right edge, followed by
//!   `\noalign{\vskip-\arrayrulewidth}` (no net space).
//! * booktabs: `\toprule`/`\midrule`/`\bottomrule` are `\vskip` (`\abovetopsep`
//!   = 0 for the top rule, `\aboverulesep` = .4ex otherwise, or
//!   `\doublerulesep` when the previous rule was also a booktabs rule), an
//!   `\hrule` of `\heavyrulewidth` (.08em) / `\lightrulewidth` (.05em), then
//!   `\belowrulesep` (.65ex; `\belowbottomsep` = 0 after `\bottomrule`) unless
//!   another booktabs rule follows. `\cmidrule[w](trim){a-b}` is
//!   `\aboverulesep` (if the previous rule class is 0), a row of `\cmidrulewidth`
//!   (.03em) leaders trimmed by `\cmidrulekern` (.5em) on the requested sides,
//!   then `\vskip-w` before another `\cmidrule` or `\belowrulesep`. The em/ex
//!   are those of the body font (the dimensions are assigned when booktabs is
//!   loaded).
//!
//! Lengths the kernel allows to change (`\tabcolsep`, `\arrayrulewidth`,
//! `\doublerulesep`) are read from `\setlength` in the source by the adapter:
//! the compiler bakes the kernel defaults into its templates, so its spaces
//! of exactly those defaults are mapped back to the document's values.

use flashtex_compiler::parser::Inline;
use flashtex_compiler::tabular::{self as ct, Align, BookRule, Length, VerticalPosition};
use flashtex_compiler::Span;

use crate::adapter::Item;

/// The measure an unbreakable entry is set in: just under TeX's `\maxdimen`
/// (16383.99998pt), which paragraph-layout rejects as a line width.
pub const MAX_DIMEN_PT: f64 = 16383.0;
/// cmsy `axis_height` (Latin Modern/Computer Modern) in em.
pub const AXIS_EM: f64 = 0.25;
/// booktabs widths (em) and separations (ex / em).
const HEAVY_RULE_EM: f64 = 0.08;
const LIGHT_RULE_EM: f64 = 0.05;
const CMID_RULE_EM: f64 = 0.03;
const ABOVE_RULE_SEP_EX: f64 = 0.4;
const BELOW_RULE_SEP_EX: f64 = 0.65;
const CMID_RULE_KERN_EM: f64 = 0.5;

/// A dimension rounded to TeX's scaled points.
pub fn sp(pt: f64) -> f64 {
    (pt * 65536.0).round() / 65536.0
}

/// `\baselineskip` of a size declaration under the 10/11/12pt class options
/// (size10.clo/size11.clo/size12.clo `\@setfontsize`), keyed by the font size
/// in hundredths of a point as the adapter declares it. Unknown sizes use
/// 1.2 times the size.
pub fn baselineskip_pt(class_size: u32, size_cpt: u16) -> f64 {
    let table: &[(u16, f64)] = match class_size {
        12 => &[(600, 7.0), (800, 9.5), (1000, 12.0), (1095, 13.6), (1200, 14.5), (1440, 18.0), (1728, 22.0), (2074, 25.0), (2488, 30.0)],
        11 => &[(600, 7.0), (800, 9.5), (900, 11.0), (1000, 12.0), (1095, 13.6), (1200, 14.0), (1440, 18.0), (1728, 22.0), (2074, 25.0), (2488, 30.0)],
        _ => &[(500, 6.0), (700, 8.0), (800, 9.5), (900, 11.0), (1000, 12.0), (1200, 14.0), (1440, 18.0), (1728, 22.0), (2074, 25.0), (2488, 30.0)],
    };
    table.iter().find(|(s, _)| *s == size_cpt).map_or(f64::from(size_cpt) / 100.0 * 1.2, |(_, b)| *b)
}

pub fn resolve(length: Length, measure: f64) -> f64 {
    match length {
        Length::Pt(pt) => pt,
        Length::TextWidth(factor) => factor * measure,
    }
}

/// The kernel lengths a document may change with `\setlength`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableLengths {
    pub tabcolsep: f64,
    pub arrayrulewidth: f64,
    pub doublerulesep: f64,
}

impl Default for TableLengths {
    fn default() -> Self {
        TableLengths {
            tabcolsep: ct::TABCOLSEP_PT,
            arrayrulewidth: ct::ARRAYRULEWIDTH_PT,
            doublerulesep: ct::DOUBLERULESEP_PT,
        }
    }
}

impl TableLengths {
    /// Reads each length through `value` (a `\setlength` lookup), keeping
    /// the kernel default when the document does not set it.
    pub fn read(mut value: impl FnMut(&str) -> Option<f64>) -> TableLengths {
        let d = TableLengths::default();
        TableLengths {
            tabcolsep: value("tabcolsep").unwrap_or(d.tabcolsep),
            arrayrulewidth: value("arrayrulewidth").unwrap_or(d.arrayrulewidth),
            doublerulesep: value("doublerulesep").unwrap_or(d.doublerulesep),
        }
    }

    /// A compiler template space, which is one of the kernel defaults the
    /// preamble builder inserts, at this document's value.
    fn space(&self, pt: f64) -> f64 {
        if pt == ct::TABCOLSEP_PT {
            self.tabcolsep
        } else if pt == ct::DOUBLERULESEP_PT {
            self.doublerulesep
        } else if pt == ct::ARRAYRULEWIDTH_PT / 2.0 {
            self.arrayrulewidth / 2.0
        } else {
            pt
        }
    }
}

/// A table as the pipeline sets it: the compiler's structure with every
/// inline list already converted to pipeline items.
#[derive(Debug, Clone, PartialEq)]
pub struct TableItem {
    pub columns: Vec<TableColumn>,
    pub entries: Vec<TableEntry>,
    pub position: VerticalPosition,
    pub width: Option<Length>,
    pub arraystretch: f64,
    /// Size declaration in force at `\begin` (hundredths of a point; 0 keeps
    /// the surrounding size).
    pub size_cpt: u16,
    pub lengths: TableLengths,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub before: Vec<TableMaterial>,
    pub align: Align,
    pub after: Vec<TableMaterial>,
    /// `\extracolsep{\fill}` `\tabskip` glue after this column.
    pub fill_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableMaterial {
    Space(f64),
    Rule(Span),
    Text(Vec<Item>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub items: Vec<Item>,
    pub columns: usize,
    pub template: Option<TableColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableEntry {
    Row { cells: Vec<TableCell>, extra_depth_pt: f64 },
    HLine { span: Span },
    CLine { first: usize, last: usize, span: Span },
    BookRule { kind: BookRule, width_pt: Option<f64>, span: Span },
    CMidRule { first: usize, last: usize, trim_left: bool, trim_right: bool, width_pt: Option<f64>, span: Span },
    VSpace { pt: f64 },
}

fn material(m: &[ct::Material], lengths: TableLengths, items_of: &mut dyn FnMut(&[Inline]) -> Vec<Item>) -> Vec<TableMaterial> {
    let mut out = Vec::with_capacity(m.len());
    for m in m {
        out.push(match m {
            ct::Material::Space(pt) => TableMaterial::Space(lengths.space(*pt)),
            ct::Material::Rule(span) => TableMaterial::Rule(*span),
            ct::Material::Text(inlines) => TableMaterial::Text(items_of(inlines)),
        });
    }
    out
}

fn column(t: &ct::ColumnTemplate, lengths: TableLengths, items_of: &mut dyn FnMut(&[Inline]) -> Vec<Item>) -> TableColumn {
    TableColumn {
        before: material(&t.before, lengths, items_of),
        align: t.align,
        after: material(&t.after, lengths, items_of),
        fill_after: t.fill_after,
    }
}

/// Converts the compiler's table, turning every inline list into items with
/// `items_of`.
pub fn from_compiler(t: &ct::Tabular, lengths: TableLengths, size_cpt: u16, items_of: &mut dyn FnMut(&[Inline]) -> Vec<Item>) -> TableItem {
    let columns = t.columns.iter().map(|c| column(c, lengths, items_of)).collect();
    let mut entries = Vec::with_capacity(t.entries.len());
    for e in &t.entries {
        entries.push(match e {
            ct::Entry::Row(row) => TableEntry::Row {
                cells: row
                    .cells
                    .iter()
                    .map(|c| TableCell {
                        items: items_of(&c.content),
                        columns: c.columns,
                        template: c.template.as_ref().map(|tp| column(tp, lengths, items_of)),
                    })
                    .collect(),
                extra_depth_pt: row.extra_depth_pt,
            },
            ct::Entry::HLine { span } => TableEntry::HLine { span: *span },
            ct::Entry::CLine { first, last, span } => TableEntry::CLine { first: *first, last: *last, span: *span },
            ct::Entry::BookRule { kind, width_pt, span } => TableEntry::BookRule { kind: *kind, width_pt: *width_pt, span: *span },
            ct::Entry::CMidRule { first, last, trim_left, trim_right, width_pt, span } => TableEntry::CMidRule {
                first: *first,
                last: *last,
                trim_left: *trim_left,
                trim_right: *trim_right,
                width_pt: *width_pt,
                span: *span,
            },
            ct::Entry::VSpace { pt } => TableEntry::VSpace { pt: *pt },
        });
    }
    TableItem {
        columns,
        entries,
        position: t.position,
        width: t.width,
        arraystretch: t.arraystretch,
        size_cpt,
        lengths,
        span: t.span,
    }
}

/// Width, height and depth of a set box.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dims {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

/// A measured `u`/`v` template piece.
#[derive(Debug, Clone, PartialEq)]
pub enum MPiece {
    Space(f64),
    Rule(Span),
    Text(Dims),
}

impl MPiece {
    fn width(&self) -> f64 {
        match self {
            MPiece::Space(pt) => *pt,
            MPiece::Rule(_) => 0.0,
            MPiece::Text(d) => d.width,
        }
    }
}

/// A measured entry: template pieces around the entry box.
#[derive(Debug, Clone, PartialEq)]
pub struct MCell {
    pub column: usize,
    pub columns: usize,
    pub align: Align,
    pub before: Vec<MPiece>,
    /// The entry: an hbox for `l`/`c`/`r`, the `\vtop` for `p{}`.
    pub content: Dims,
    pub after: Vec<MPiece>,
}

impl MCell {
    fn natural(&self) -> f64 {
        self.before.iter().map(MPiece::width).sum::<f64>() + self.content.width + self.after.iter().map(MPiece::width).sum::<f64>()
    }
}

/// The columns each cell of a row occupies and its template (`None` when
/// the row has more cells than the preamble has columns: TeX's "extra
/// alignment tab" recovery, which the compiler already diagnosed).
pub fn row_slots<'t>(table: &'t TableItem, cells: &'t [TableCell]) -> Vec<(usize, usize, Option<&'t TableColumn>)> {
    let n = table.columns.len().max(1);
    let mut out = Vec::with_capacity(cells.len());
    let mut column = 0;
    for cell in cells {
        if column >= n {
            break;
        }
        let columns = cell.columns.clamp(1, n - column);
        out.push((column, columns, cell.template.as_ref().or_else(|| table.columns.get(column))));
        column += columns;
    }
    out
}

/// Which part of a cell a placed box is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Before(usize),
    Content,
    After(usize),
}

/// A box placed in the table: `baseline` is its reference point's y below
/// the table's baseline (negative above), `x` its left edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub row: usize,
    pub cell: usize,
    pub slot: Slot,
    pub x: f64,
    pub baseline: f64,
}

/// A filled rule; `top` is relative to the table's baseline (y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedRule {
    pub x: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub placed: Vec<Placed>,
    pub rules: Vec<PlacedRule>,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

/// Font- and page-dependent values the geometry needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub strut_height: f64,
    pub strut_depth: f64,
    /// Body font quad and x-height (booktabs' em/ex).
    pub em: f64,
    pub ex: f64,
    /// Math axis height at the table's size (`\vcenter`).
    pub axis: f64,
    /// `\textwidth`, for `p{.5\textwidth}` and `tabular*{\textwidth}`.
    pub measure: f64,
}

/// Lays the table out. `rows[i]` holds the measured cells of the i-th
/// `Row` entry, in the order [`row_slots`] returns them.
pub fn layout(table: &TableItem, rows: &[Vec<MCell>], m: &Metrics) -> Geometry {
    let n = table.columns.len().max(1);
    let arw = table.lengths.arrayrulewidth;
    let dbl = table.lengths.doublerulesep;

    // TeX §801: w[k][j] is the widest entry spanning columns k..=j.
    let mut w = vec![vec![f64::NEG_INFINITY; n]; n];
    for cell in rows.iter().flatten() {
        let first = cell.column.min(n - 1);
        let last = (cell.column + cell.columns.max(1) - 1).min(n - 1);
        w[first][last] = w[first][last].max(cell.natural());
    }
    let mut widths = vec![0.0f64; n];
    let mut tabskip = vec![0.0f64; n];
    let mut fill = vec![false; n];
    for k in 0..n {
        let has_entries = w[k][k] > f64::NEG_INFINITY;
        widths[k] = if has_entries { w[k][k] } else { 0.0 };
        fill[k] = has_entries && k + 1 < n && table.columns.get(k).is_some_and(|c| c.fill_after);
        if k + 1 < n {
            for j in k + 1..n {
                if w[k][j] > f64::NEG_INFINITY {
                    let excess = w[k][j] - widths[k] - tabskip[k];
                    w[k + 1][j] = w[k + 1][j].max(excess);
                }
            }
        }
    }
    let natural: f64 = widths.iter().sum::<f64>() + tabskip.iter().sum::<f64>();
    let box_width = match table.width {
        Some(len) => {
            let target = resolve(len, m.measure);
            let fills = fill.iter().filter(|f| **f).count();
            if fills > 0 && target > natural {
                let share = (target - natural) / fills as f64;
                for (skip, f) in tabskip.iter_mut().zip(&fill) {
                    if *f {
                        *skip += share;
                    }
                }
            }
            target
        }
        None => natural,
    };
    let mut column_x = vec![0.0f64; n + 1];
    for k in 0..n {
        column_x[k + 1] = column_x[k] + widths[k] + tabskip[k];
    }
    let right_of = |k: usize| column_x[k] + widths[k];

    let mut placed = Vec::new();
    let mut rules: Vec<PlacedRule> = Vec::new();
    let mut vrules: Vec<(f64, f64, f64, Span)> = Vec::new();
    let mut y = 0.0f64;
    let mut first_height: Option<f64> = None;
    let mut last_depth = 0.0;
    let mut last_rule_class = 0u8;
    let mut row_index = 0usize;
    let rule = |rules: &mut Vec<PlacedRule>, x: f64, top: f64, width: f64, height: f64, span: Span| {
        if width > 0.0 && height > 0.0 {
            rules.push(PlacedRule { x, top, width, height, span });
        }
    };
    for (index, entry) in table.entries.iter().enumerate() {
        let next = table.entries.get(index + 1);
        let next_is_booktabs = matches!(next, Some(TableEntry::BookRule { .. } | TableEntry::CMidRule { .. }));
        last_depth = 0.0;
        match entry {
            TableEntry::Row { extra_depth_pt, .. } => {
                let ri = row_index;
                row_index += 1;
                let cells = rows.get(ri).map_or(&[][..], Vec::as_slice);
                let mut height = m.strut_height;
                let mut depth = m.strut_depth;
                if *extra_depth_pt > 0.0 {
                    depth = depth.max(m.strut_depth + extra_depth_pt);
                }
                for cell in cells {
                    height = height.max(cell.content.height);
                    depth = depth.max(cell.content.depth);
                    for piece in cell.before.iter().chain(&cell.after) {
                        if let MPiece::Text(d) = piece {
                            height = height.max(d.height);
                            depth = depth.max(d.depth);
                        }
                    }
                }
                first_height.get_or_insert(height);
                let top = y;
                let baseline = y + height;
                for (ci, cell) in cells.iter().enumerate() {
                    let first = cell.column.min(n - 1);
                    let last = (cell.column + cell.columns.max(1) - 1).min(n - 1);
                    let left = column_x[first];
                    let right = right_of(last);
                    let mut place = |piece: &MPiece, x: f64, slot: Slot, placed: &mut Vec<Placed>| match piece {
                        MPiece::Space(_) => {}
                        MPiece::Rule(span) => vrules.push((x - arw / 2.0, top, top + height + depth, *span)),
                        MPiece::Text(_) => placed.push(Placed { row: ri, cell: ci, slot, x, baseline }),
                    };
                    let mut x = left;
                    for (pi, piece) in cell.before.iter().enumerate() {
                        place(piece, x, Slot::Before(pi), &mut placed);
                        x += piece.width();
                    }
                    let after_width: f64 = cell.after.iter().map(MPiece::width).sum();
                    let content_x = match cell.align {
                        Align::Left | Align::Paragraph(_) => x,
                        Align::Right => right - after_width - cell.content.width,
                        Align::Center => x + (right - after_width - x - cell.content.width) / 2.0,
                    };
                    placed.push(Placed { row: ri, cell: ci, slot: Slot::Content, x: content_x, baseline });
                    let mut x = right - after_width;
                    for (pi, piece) in cell.after.iter().enumerate() {
                        place(piece, x, Slot::After(pi), &mut placed);
                        x += piece.width();
                    }
                }
                y += height + depth;
                last_depth = depth;
            }
            TableEntry::HLine { span } => {
                first_height.get_or_insert(arw);
                rule(&mut rules, 0.0, y, box_width, arw, *span);
                y += arw;
                if matches!(next, Some(TableEntry::HLine { .. })) {
                    y += dbl - arw;
                }
            }
            TableEntry::CLine { first, last, span } => {
                first_height.get_or_insert(arw);
                let (first, last) = ((*first).min(n - 1), (*last).min(n - 1));
                let x = column_x[first];
                rule(&mut rules, x, y, right_of(last) - x, arw, *span);
            }
            TableEntry::VSpace { pt } => {
                first_height.get_or_insert(0.0);
                y += pt;
            }
            TableEntry::BookRule { kind, width_pt, span } => {
                let width = width_pt.unwrap_or(match kind {
                    BookRule::Mid => LIGHT_RULE_EM * m.em,
                    BookRule::Top | BookRule::Bottom => HEAVY_RULE_EM * m.em,
                });
                let above = match kind {
                    BookRule::Top => 0.0,
                    BookRule::Mid | BookRule::Bottom => ABOVE_RULE_SEP_EX * m.ex,
                };
                // `\@BTrule` always appends its `\vskip` first, so a `[t]`
                // table opening with a booktabs rule has zero height.
                first_height.get_or_insert(0.0);
                y += if last_rule_class == 0 { above } else { dbl };
                rule(&mut rules, 0.0, y, box_width, width, *span);
                y += width;
                if next_is_booktabs {
                    last_rule_class = 1;
                } else {
                    last_rule_class = 0;
                    if *kind != BookRule::Bottom {
                        y += BELOW_RULE_SEP_EX * m.ex;
                    }
                }
            }
            TableEntry::CMidRule { first, last, trim_left, trim_right, width_pt, span } => {
                let width = width_pt.unwrap_or(CMID_RULE_EM * m.em);
                if last_rule_class == 0 {
                    first_height.get_or_insert(0.0);
                    y += ABOVE_RULE_SEP_EX * m.ex;
                } else {
                    first_height.get_or_insert(width);
                }
                let (first, last) = ((*first).min(n - 1), (*last).min(n - 1));
                let kern = CMID_RULE_KERN_EM * m.em;
                let left = column_x[first] + if *trim_left { kern } else { 0.0 };
                let right = right_of(last) - if *trim_right { kern } else { 0.0 };
                rule(&mut rules, left, y, right - left, width, *span);
                y += width;
                if matches!(next, Some(TableEntry::CMidRule { .. })) {
                    y -= width;
                    last_rule_class = 1;
                } else {
                    y += BELOW_RULE_SEP_EX * m.ex;
                    last_rule_class = 0;
                }
            }
        }
    }

    // One rule per `|` over consecutive rows it runs through.
    vrules.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut merged: Vec<(f64, f64, f64, Span)> = Vec::new();
    for r in vrules {
        match merged.last_mut() {
            Some(last) if last.0 == r.0 && last.3 == r.3 && (last.2 - r.1).abs() < 1e-9 => last.2 = r.2,
            _ => merged.push(r),
        }
    }
    for (x, top, bottom, span) in merged {
        rule(&mut rules, x, top, arw, bottom - top, span);
    }

    let total = y;
    let reference = match table.position {
        VerticalPosition::Top => first_height.unwrap_or(0.0),
        VerticalPosition::Bottom => total - last_depth,
        VerticalPosition::Center => total / 2.0 + m.axis,
    };
    for p in &mut placed {
        p.baseline -= reference;
    }
    for r in &mut rules {
        r.top -= reference;
    }
    Geometry {
        placed,
        rules,
        width: box_width,
        height: reference,
        depth: total - reference,
    }
}

#[cfg(test)]
mod tests {
    //! Font-free geometry checks against kernel arithmetic (12pt body:
    //! `\baselineskip` 14.5pt, strut 10.15pt + 4.35pt).
    use super::*;

    fn span() -> Span {
        Span::new(0, 1)
    }

    fn col(align: Align, rule_before: bool, rule_after: bool) -> TableColumn {
        let mut before = Vec::new();
        if rule_before {
            before.push(TableMaterial::Rule(span()));
        }
        before.push(TableMaterial::Space(6.0));
        let mut after = vec![TableMaterial::Space(6.0)];
        if rule_after {
            after.push(TableMaterial::Rule(span()));
        }
        TableColumn { before, align, after, fill_after: false }
    }

    fn table(columns: Vec<TableColumn>, entries: Vec<TableEntry>) -> TableItem {
        TableItem {
            columns,
            entries,
            position: VerticalPosition::Top,
            width: None,
            arraystretch: 1.0,
            size_cpt: 0,
            lengths: TableLengths::default(),
            span: span(),
        }
    }

    fn row(n: usize) -> TableEntry {
        TableEntry::Row {
            cells: (0..n).map(|_| TableCell { items: Vec::new(), columns: 1, template: None }).collect(),
            extra_depth_pt: 0.0,
        }
    }

    fn measured(t: &TableItem, widths: &[&[f64]]) -> Vec<Vec<MCell>> {
        let mut rows = Vec::new();
        let mut wi = 0;
        for e in &t.entries {
            let TableEntry::Row { cells, .. } = e else { continue };
            let ws = widths[wi];
            wi += 1;
            rows.push(
                row_slots(t, cells)
                    .into_iter()
                    .zip(ws)
                    .map(|((column, columns, tp), w)| {
                        let piece = |m: &TableMaterial| match m {
                            TableMaterial::Space(pt) => MPiece::Space(*pt),
                            TableMaterial::Rule(s) => MPiece::Rule(*s),
                            TableMaterial::Text(_) => MPiece::Text(Dims::default()),
                        };
                        MCell {
                            column,
                            columns,
                            align: tp.map_or(Align::Left, |c| c.align),
                            before: tp.map_or(Vec::new(), |c| c.before.iter().map(piece).collect()),
                            content: Dims { width: *w, height: 8.0, depth: 2.0 },
                            after: tp.map_or(Vec::new(), |c| c.after.iter().map(piece).collect()),
                        }
                    })
                    .collect(),
            );
        }
        rows
    }

    fn metrics() -> Metrics {
        Metrics { strut_height: 10.15, strut_depth: 4.35, em: 11.74988, ex: 5.16, axis: 3.0, measure: 469.75 }
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "expected {b}, got {a}");
    }

    #[test]
    fn rules_take_no_width_and_hlines_span_the_table() {
        let t = table(
            vec![col(Align::Left, true, true), col(Align::Center, false, true), col(Align::Right, false, true)],
            vec![TableEntry::HLine { span: span() }, row(3), TableEntry::HLine { span: span() }],
        );
        let g = layout(&t, &measured(&t, &[&[10.0, 20.0, 30.0]]), &metrics());
        close(g.width, 96.0);
        let h: Vec<_> = g.rules.iter().filter(|r| r.width > 1.0).collect();
        close(h[1].top - h[0].top, 0.4 + 14.5);
        let v: Vec<_> = g.rules.iter().filter(|r| r.width < 1.0).collect();
        assert_eq!(v.len(), 4);
        for (r, x) in v.iter().zip([0.0, 22.0, 54.0, 96.0]) {
            close(r.x, x - 0.2);
            close(r.height, 14.5);
        }
        // [t]: the first item is the rule.
        close(g.height, 0.4);
    }

    #[test]
    fn multicolumn_excess_goes_to_the_last_spanned_column() {
        let mut t = table(vec![col(Align::Left, false, false), col(Align::Left, false, false)], vec![row(1), row(2)]);
        if let TableEntry::Row { cells, .. } = &mut t.entries[0] {
            cells[0].columns = 2;
            cells[0].template = Some(col(Align::Center, false, false));
        }
        let g = layout(&t, &measured(&t, &[&[100.0], &[10.0, 5.0]]), &metrics());
        close(g.width, 112.0);
        let second = g.placed.iter().find(|p| p.row == 1 && p.cell == 1 && p.slot == Slot::Content).unwrap();
        close(second.x, 22.0 + 6.0);
    }

    #[test]
    fn double_hline_and_booktabs_spacing() {
        let t = table(
            vec![col(Align::Left, false, false)],
            vec![TableEntry::HLine { span: span() }, TableEntry::HLine { span: span() }, row(1)],
        );
        let g = layout(&t, &measured(&t, &[&[5.0]]), &metrics());
        close(g.rules[1].top - g.rules[0].top, 2.0);
        let m = metrics();
        let t = table(
            vec![col(Align::Left, false, false)],
            vec![
                TableEntry::BookRule { kind: BookRule::Top, width_pt: None, span: span() },
                row(1),
                TableEntry::BookRule { kind: BookRule::Mid, width_pt: None, span: span() },
                row(1),
                TableEntry::BookRule { kind: BookRule::Bottom, width_pt: None, span: span() },
            ],
        );
        let g = layout(&t, &measured(&t, &[&[5.0], &[5.0]]), &m);
        close(g.rules[0].height, 0.08 * m.em);
        let first = g.placed.iter().find(|p| p.row == 0).unwrap();
        close(first.baseline - g.rules[0].top, 0.08 * m.em + 0.65 * m.ex + 10.15);
        close(g.rules[1].top - first.baseline, 4.35 + 0.4 * m.ex);
        close(g.depth + g.height, g.rules[2].top + g.rules[2].height + g.height);
    }

    #[test]
    fn baselineskips_follow_the_size_files() {
        assert_eq!(baselineskip_pt(12, 1200), 14.5);
        assert_eq!(baselineskip_pt(12, 1095), 13.6);
        assert_eq!(baselineskip_pt(11, 1095), 13.6);
        assert_eq!(baselineskip_pt(10, 1000), 12.0);
    }
}
