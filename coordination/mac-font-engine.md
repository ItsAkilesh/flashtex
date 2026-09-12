Agent / task / branch: mac-font-engine (Claude Code subagent, parent mac-claude-a, mac-m1max-a) / FT-018 rev 1 / agent/mac-font-engine/tex-fonts
State: ready for integration (library only; follow-ups listed)
Owned paths: crates/font-engine/**, coordination/mac-font-engine.md, coordination/agents/mac-font-engine.json
Main integrated through: 984fa28f2537feadb9f848eb90f69de0327d1fa5 (branch base; origin/main a77e697 reviewed, no overlap with owned paths — see below)

Ready behavior (crates/font-engine, edition 2024, zero external crates):
- TrueType/OpenType glyf parsing and OpenType CFF (OTTO, Latin Modern): head,
  hhea, hmtx, maxp, loca/glyf or raw `CFF `, cmap 4/12, OS/2, post, name, MATH;
  .ttc by face index; variable fonts and Type 1 rejected explicitly.
- Kerning: GPOS `kern` PairPos 1/2 (+Extension), legacy kern format 0, AFM KPX.
  Feature selection through ScriptList DFLT/latn default LangSys.
- Ligatures: GSUB `liga` LigatureSubst (+Extension) applied per lookup pass
  (two-step ffi works), cmap U+FB00..04 fallback, never on fixed-pitch faces.
- Adobe Core 14 tables (Times x4, Helvetica, Courier, Symbol) generated from
  the AFMs by tools/gen_tables.py, SHA-256 provenance recorded, byte-reproducible;
  Times-Roman values equal compiler metrics.rs on de1020c ("Hello" 12pt = 26.664).
- shape(): clusters with source byte ranges + per-cluster text (ActualText),
  canonical mark composition, missing-glyph list, fail-closed on bidi/joining/
  reordering scripts (Error::UnsupportedScript), unsupported-feature notes.
- Deterministic glyf subsetting with checksums and explicit old->new GID map;
  PDF data: FontFile2 subset (CIDFontType2) or whole OpenType program for CFF
  (FontFile3/OpenType, CIDFontType0, identity CIDs), /W runs, descriptor,
  ToUnicode with multi-char bfchar + parser. MATH: all 56 MathConstants,
  italics correction, top-accent attachment.
- Bounded FontSearch (explicit dir list, no scanning).

Incomplete behavior (declared in README):
- CFF subsetting (whole program embedded instead); CFF charstrings not parsed.
- MathVariants/MathKernInfo; GPOS mark attachment; per-run language tags;
  GSUB beyond `liga`; Arabic/Indic/RTL; vertical.
- Helvetica-Bold/Oblique, Courier variants, ZapfDingbats tables (one-line
  generator change).

Interface changes and required consumer actions: none on main. No runtime-v1
item kinds emitted or proposed. README "Proposed ABI (non-authoritative until
Commander accepts)" states: content-addressed FontId (SHA-256 of program bytes +
face index), ORIGINAL glyph ids in shaping output, clusters with byte ranges and
text, renumbering only inside embed with an explicit map, unit conversions,
missing/unsupported reporting, bounded resolution.

Validation: `cargo test` 45 passed (5 unit, 14 Core14, 9 Latin Modern, 17
TrueType; font-file tests skip with a message if absent); clippy and fmt clean.
CoreText comparison (examples/compare_coretext.swift, macOS 26.3.1): TrueType
and Latin Modern plain advances 0.0000 pt delta on the same file; Latin Modern
shaped lines (kerning + ligatures) 0.0000 pt delta; Core 14 tables vs Apple's
different Times/Helvetica/Courier programs within 0.05 pt on ASCII sentences
(0.09 pt on 60 Courier glyphs); itemised larger deltas (AFM Euro placeholder,
kerning-table differences) recorded in README. latinmodern-math axisHeight 250,
fractionRuleThickness 40, radicalKernAfterDegree -556 verified independently.

Needs from others: Commander review of the proposed ABI; PDF crate owner to
decide whether crates/pdf delegates subsetting/embedding here.
Next action: none pending for rev 1; follow-ups above on request.

Peer revisions reviewed and adaptations:
- de1020c (compiler metrics.rs): identical AFM widths; kept an independent
  generated copy, asserted equality in tests.
- 5b5f7b5 (crates/pdf truetype/embed): reused sfnt writer/checksum approach;
  extended with explicit maps, composite bbox, multi-char ToUnicode, CFF path.
- 30a14a6 (fontmetrics proposal): reused the per-glyph CoreText measurement
  method; replaced CoreText-derived tables with AFM-exact ones.
- origin/main a77e697: docs/contracts/rendering-v2-proposal.md requires exact
  font bytes + face index identity, original glyph ids, clusters with source
  provenance, and missing font = failure; this crate's FontId/GlyphId/Cluster/
  missing design satisfies each point (variable-font "instance" is moot: they
  are rejected). transfer-v1.md does not touch fonts. No adaptation needed.
Commit identity note: 07fa9fe was authored as jay3332 <me@jay3332.tech> (the
user's address, not the repo-configured noreply one); later commits use the
repository configuration unchanged. Not rewritten (published history).
Resource: allocation claude-mac20x-font-engine; shared Max quota unknown.
Updated: 2026-09-12T06:45:00Z
