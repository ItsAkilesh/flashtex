Agent / task / branch: mac-font-engine (Claude Code subagent, parent mac-claude-a, mac-m1max-a) / FT-018 rev 1 / agent/mac-font-engine/tex-fonts
State: in progress
Owned paths: crates/font-engine/**, coordination/mac-font-engine.md, coordination/agents/mac-font-engine.json
Main integrated through: 984fa28f2537feadb9f848eb90f69de0327d1fa5

Ready behavior:
- crates/font-engine: `flashtex-font-engine` (edition 2024, zero external crates).
- TrueType/OpenType glyf parsing (head, hhea, hmtx, maxp, loca, glyf, cmap 4/12,
  OS/2, post, name), .ttc by face index; OTTO/variable fonts rejected explicitly.
- Kerning: GPOS `kern` PairPos 1/2 (+Extension), legacy `kern` format 0, AFM KPX.
- Ligatures: GSUB `liga` LigatureSubst (+Extension) and cmap U+FB00..04 fallback.
- Adobe Core 14 tables (Times x4, Helvetica, Courier, Symbol) generated from the
  AFMs by tools/gen_tables.py with SHA-256 provenance; Times-Roman values equal
  the compiler's metrics.rs on de1020c ("Hello" 12pt = 26.664 pt exactly).
- shape(): clusters with source byte ranges and per-cluster text, mark
  composition through canonical pairs, missing-glyph list, fail-closed on
  bidi/joining/reordering scripts (Error::UnsupportedScript).
- Deterministic subsetting with checksums and explicit old->new GID map;
  ToUnicode CMap (multi-char bfchar) + parser; /W, descriptor, CIDToGID data.
- Bounded FontSearch (explicit dir list, no scanning).

Incomplete behavior:
- CoreText comparison example and README deltas (next checkpoint).
- No GPOS mark attachment, no script/language selection in feature lookup,
  no CFF, no variable fonts, no vertical, no Arabic/Indic.

Interface changes and required consumer actions: none on main; the crate is a
library for a future negotiated display format (README "Proposed ABI" pending).

Validation: `cargo test` in crates/font-engine: 30 passed (5 unit, 13 core14,
12 truetype using Apple system fonts, skip-with-message if absent).

Needs from others: Commander review of the proposed ABI once the README lands.
Next action: examples/compare_coretext.swift + examples/measure.rs, README with
provenance/licences/unsupported/ABI, clippy cleanup, report.
Peer revisions reviewed and adaptations: de1020c (compiler metrics.rs: same AFM
widths, kept independent copy); 5b5f7b5 (crates/pdf truetype/embed: reused sfnt
writer/checksum approach, extended ToUnicode to multi-char); 30a14a6
(fontmetrics proposal: reused CoreText per-glyph measurement approach).
Resource: allocation claude-mac20x-font-engine; shared Max quota unknown.
Updated: 2026-09-12T05:50:00Z
