# daniel-color handoff

Agent / task / branch: daniel-color / FT-035 (revision 4) original bounded
xcolor-style colour-expression parser / `agent/daniel-color/color-expressions`

State: ready for integration (standalone crate; nothing else on main touched)

## Revision 4: existing consumer integration fixture + measured unsupported gaps

Revision 4's objective: build an integration fixture against a *real*
already-wired downstream boundary (not a hypothetical one), and replace
revision 3's prose "we don't implement X" compatibility statement with
counted, named gaps for the two items that are actually countable (the
named-colour table and the colour-model set).

### 1. Existing consumer integration fixture — and exactly where it stops

New `crates/color-expressions/tests/vector_graphics_consumer.rs` (5 tests).
First, the boundary search: this crate already resolves into
`flashtex_vector_graphics::Color`/`Paint`, so the question was who actually
paints with those types. Grepping every `crates/*/Cargo.toml` for
`flashtex-vector-graphics` finds exactly **two** dependents: this crate and
`vector-graphics` itself. `crates/pdf` and `crates/rendering-core` — the two
crates that actually write PDF bytes — have **zero** Cargo dependency edge
to `flashtex-vector-graphics`, and each already has its own, different,
non-interconvertible colour representation that nothing today converts
to/from `flashtex_vector_graphics::Color`:

- `crates/pdf/src/v2.rs`: `Paint { rgb: Option<[Decimal; 3]> }` — fixed-point
  decimal, RGB-only, no gray/CMYK case.
- `crates/rendering-core/src/pdf_stream.rs` and `.../src/mixed.rs`: a JSON
  `paint` object shaped `{"r":_,"g":_,"b":_,"a":_}` under a hard-coded
  `"color_space":"srgb"`.

This matches what `crates/vector-graphics/src/lib.rs`'s own module doc
already says: it is "a proposal for later consumer integration
(rendering-v2, after ABI agreement) ... nothing here is wired into the
compiler, the Mac shell, or `crates/pdf` yet" — confirmed structurally here,
not just taken on faith from the comment.

So the honest fixture is this: prove resolved `Color`/`Paint` values survive
unchanged through `vector-graphics`'s own real painter-facing surface —
`Item::Rule`/`PathFill`/`PathStroke` inside a `DisplayList`, fed to
`pdf::content_stream` (the function that actually emits PDF content-stream
operator bytes: `rg`/`RG`, `g`/`G`, `k`/`K`, `/GSn gs` with `ca`/`CA`). It
cannot prove more than that — there is no code path today by which a value
this crate resolves reaches an actual `crates/pdf`-written file or a
`crates/rendering-core` display list, because neither crate consumes this
dependency chain at all. That is the boundary's actual stopping point, not
a simplification for this handoff.

Five tests, every expected string/value hand-derived from `pdf.rs`'s own
documented `num()` (3 decimals, trailing zeros trimmed) and flip
(`bottom = H - y - height`) rules — never produced by calling this crate's
own resolver:

- `resolved_rgb_mix_survives_into_a_real_pdf_fill_operator`: `red!50!blue`
  -> `Rgb(0.5,0,0.5)` -> `Rule` in a 100x100pt page -> `content_stream` ->
  `"0.5 0 0.5 rg\n0 50 50 50 re f\n"`.
- `resolved_cmyk_literal_survives_into_a_real_pdf_fill_operator`:
  `cmyk:0.25,0.5,0.75,1` -> `"0.25 0.5 0.75 1 k\n10 40 30 40 re f\n"`.
- `resolved_gray_literal_survives_into_a_real_pdf_stroke_operator`:
  `gray:0.5` through a `PathStroke` (not a fill) -> the uppercase stroke
  operator, `"0.5 G\n2 w\n10 190 m\n100 150 l\nS\n"`.
- `resolved_alpha_survives_into_a_real_pdf_ext_g_state`: `resolve_paint`'s
  straight alpha specifically (not the colour) -> a real `ExtGState{name:
  "GS0", alpha: 0.3}` and its `"<< /Type /ExtGState /ca 0.3 /CA 0.3 >>"`
  dictionary.
- `resolved_negated_mix_through_custom_palette_survives_the_boundary`: a
  composed expression (`-brandA!50!brandB`) through a non-base, custom
  `Palette` -> `Rgb(0.125, 0.5, 0.875)` -> the same real fill path, proving
  the boundary holds for composed expressions, not just single literals.

### 2. Measured unsupported gaps

Revision 3 listed these as prose ("xcolor's named colour tables", "HTML,
RGB255 and hsb models") without counting them. Measured here instead, from
xcolor's own documented base-colour list and colour-model classification
(both independently confirmed from xcolor's manual/source, not guessed):

**Named colours** — xcolor's base set (available with *no* package option
loaded) is exactly 19 names: `black, blue, brown, cyan, darkgray, gray,
green, lightgray, lime, magenta, olive, orange, pink, purple, red, teal,
violet, white, yellow`. This crate's `base_palette()` has 9 entries.

| Covered — 9 of 19 | Missing by name — 10 of 19 |
|---|---|
| black, blue, cyan, gray, green, magenta, red, white, yellow | brown, darkgray, lightgray, lime, olive, orange, pink, purple, teal, violet |

So `base_palette()` covers **9/19 (47%)** of xcolor's base names and is
missing the other 10, named above. (This compares only xcolor's *base* set
with no options; the `dvipsnames`/`svgnames`/`x11names` option tables add
several hundred further names this crate covers none of — a different,
much larger gap, not counted into the 19/9/10 figures above so the
comparison stays apples-to-apples.)

**Colour models** — xcolor documents 14 named colour-model identifiers
across four categories:

| Category | Models | Count |
|---|---|---|
| Core (`0..1` float, used for calculation) | `gray, rgb, cmy, cmyk, hsb` | 5 |
| Integer (finite-integer input, converted into a core model) | `RGB, HTML, HSB, Gray` | 4 |
| Decimal (special-purpose float variants of `hsb`/spectral) | `Hsb, tHsb, wave` | 3 |
| Pseudo (one-directional, "calculation-averse") | `named, ps` | 2 |
| **Total** | | **14** |

This crate's `model:components` literal syntax accepts exactly **3 of the
14** — `gray`, `rgb`, `cmyk` — all three from the core category, matching
exactly the three variants `flashtex_vector_graphics::Color` has. It
rejects the other **11**, every one as `ColorExprError::UnsupportedColorModel`
with no partial or approximate handling: `cmy`, `hsb` (core); `RGB`, `HTML`,
`HSB`, `Gray` (integer); `Hsb`, `tHsb`, `wave` (decimal); `named`, `ps`
(pseudo). Model matching is exact-string, case-sensitive: `RGB:255,0,0` and
`Gray:8` are rejected as unrecognised names distinct from this crate's own
lowercase `rgb`/`gray`, never treated as case-insensitive spellings of the
accepted ones. (This crate's bare `Name(String)` atom — plain palette
lookup like `red` — is a different mechanism from xcolor's `named`
pseudo-model, xcolor's own colour-table-aliasing device; the two are not
being conflated or claimed equivalent here.)

**Everything else revision 3 listed** is one specific behaviour each, not a
countable set, so it is restated at its exact existing scope rather than
re-measured: hue-aware `-color` negation (this crate's negation is a plain
per-channel `1 - x` complement, never HSB-based hue rotation — see
`expr::negate`); non-integer or out-of-`0..=100` mix weights (`left!pct!right`
requires `pct` to be an integer in `0..=100`, always `InvalidPercentage`
otherwise — xcolor additionally accepts decimal weights and, via its
extended syntax, weights outside `0..100`).

No grammar or semantics changed this revision — only the new integration
fixture, this measured write-up, and `coordination/agents/daniel-color.json`.

## Revision 3: adversarial bounds + exact identity regressions

Revision 3's objective: attack the parser with hostile expressions (every one
must be a typed error, never a panic/hang/unbounded allocation, asserted by
exact variant) and pin resolved colour values for a representative
expression set as literal expected components, keeping the rev-2 discipline
of hand-derived expectations. No grammar or semantics changed this
revision — only tests, plus this handoff's explicit compatibility statement.

### Adversarial bounds added (all exact-variant `assert_eq!`, in
`src/parser.rs` unless noted; a handful of end-to-end propagation checks are
in `src/lib.rs`)

- **Paren depth, right at and past the bound**:
  `paren_depth_exactly_at_bound_succeeds` (`MAX_DEPTH - 1` levels, `Ok`),
  `paren_depth_one_past_bound_is_too_deep` (`MAX_DEPTH` levels,
  `TooDeep { max: MAX_DEPTH }`). The existing rev-1/2
  `rejects_deeply_nested_parens_with_typed_error` (`MAX_DEPTH + 5`) is kept
  as a further-past-bound case.
- **Negation chain, right at and past**:
  `negation_chain_exactly_at_bound_succeeds` /
  `negation_chain_one_past_bound_is_too_deep`, same `TooDeep` variant.
- **Mix chain, right at and past**: `mix_chain_exactly_at_bound_succeeds` /
  `mix_chain_one_past_bound_is_too_deep`, same `TooDeep` variant.
- **512-byte cap, right at and past**:
  `input_exactly_at_byte_cap_is_not_too_long` (exactly `MAX_INPUT_LEN`
  bytes, `Ok`), `input_one_byte_past_cap_is_too_long`
  (`TooLong { len: MAX_INPUT_LEN + 1, max: MAX_INPUT_LEN }`); mirrored end
  to end as `resolve_propagates_too_long_unchanged` in `lib.rs`.
- **Percentage 0, 100, 101, huge**: `percentage_zero_is_valid` /
  `percentage_one_hundred_is_valid` (both `Ok`, exact AST),
  `percentage_101_is_invalid_by_one`
  (`InvalidPercentage { pos: 4, text: "101" }`),
  `percentage_huge_number_overflowing_u32_is_invalid_percentage_not_a_panic`
  (11-nines string past `u32::MAX`, same `InvalidPercentage` variant — the
  `digits.parse::<u32>()` failure path, never a wraparound or a panic);
  mirrored end to end as `resolve_propagates_huge_percentage_as_invalid_percentage_not_a_panic`.
- **Repeated negation**: covered by the negation-chain bound tests above
  (both the boundary pair and the original rev-1 `MAX_DEPTH + 5` case).
- **Unterminated parenthesis**: `unterminated_open_paren_is_unexpected_end`
  (`"(red"` -> `UnexpectedEnd`); mirrored end to end as
  `resolve_propagates_unterminated_paren_as_unexpected_end`.
- **Empty model component list**: `empty_model_component_list_is_unexpected_end`,
  parametrised over `gray:`/`rgb:`/`cmyk:` with nothing after the colon, all
  `UnexpectedEnd`; mirrored end to end (`rgb:` case) as
  `resolve_propagates_empty_component_list_as_unexpected_end`.
- **Component count wrong for the model**: existing rev-2
  `wrong_component_count_is_a_typed_error` (rgb/gray) plus new
  `wrong_component_count_for_cmyk_is_a_typed_error`, all
  `InvalidComponentCount { model, expected, found }`.
- **Components out of range**: existing rev-2
  `component_over_one_is_out_of_range` (`ComponentOutOfRange`), plus new
  `component_with_leading_minus_sign_is_unexpected_char_not_out_of_range`
  (a leading `-` is never parsed as a negative number — it fails at the
  first character of the component as `UnexpectedChar`, a *different*
  variant from an in-range-but-too-large value, asserted as such) and
  `component_boundary_values_zero_and_one_are_valid` (`0` and `1` exactly,
  both `Ok`).
- **Lone separator**: `lone_bang_is_unexpected_char` (`"!"` alone),
  `lone_comma_at_top_level_is_unexpected_char` (`","` alone),
  `lone_comma_as_first_component_is_unexpected_char` (`"rgb:,"`), all
  `UnexpectedChar` at the exact byte offset; mirrored end to end as
  `resolve_propagates_lone_separator_as_unexpected_char`.
- **Multi-byte characters at scanning-position boundaries**: Rust's `&str`
  is already guaranteed valid UTF-8 by the type system, so a genuinely
  invalid byte sequence can never reach this crate's public API (`resolve`/
  `resolve_paint` take `&str`, not `&[u8]`) — there is no unsafe
  byte-smuggling path to construct that case, and adding one just to test
  it would mean shipping unsafe code nobody else needs. The adjacent,
  actually-constructible attack is covered instead: a multi-byte character
  sitting exactly on a byte-length or depth-budget boundary.
  `multibyte_char_pushing_input_one_byte_past_cap_is_too_long_not_a_panic`
  puts a 4-byte character's tail past `MAX_INPUT_LEN` (`TooLong`, and the
  length check is a whole-string `len()` compare, never a slice at byte
  512, so it cannot split the character).
  `multibyte_char_immediately_past_depth_bound_is_too_deep_not_panic` puts
  one past `MAX_DEPTH` dashes before a 🎨 (`TooDeep` — `spend_depth` rejects
  before the character is even peeked).
  `multibyte_char_right_after_open_paren_reports_correct_byte_offset` pins
  the exact byte offset (`pos: 1`, not 0 and not mid-character) reported
  next to a multi-byte character. Also mirrored end to end as
  `resolve_multibyte_char_past_depth_bound_is_too_deep_not_panic`. This is
  consistent with — not new relative to — the rev-2 unicode tests already
  in `parser.rs`/`lib.rs`, which established that all scanning is by `char`
  (via `str::chars()`/`char::len_utf8`), never raw byte slicing at an
  arbitrary offset.

### Exact identity regressions added

New `crates/color-expressions/tests/identity_regressions.rs`: 16 end-to-end
`resolve(...)` calls pinned against literal `Color` constants (plain name,
negation, same-variant mixes at 0/25/50/75/100%, mix against implicit
white, same-variant Gray mix, nested parenthesised mix, negated
parenthesised mix, a three-term left-associative mix chain, negated
literals, and a CMYK-literal/RGB-literal cross-variant mix), each with the
arithmetic shown by hand in a comment, exactly as rev 2 did for the CMYK
fixtures in `src/expr.rs`. None of these expectations come from calling
`resolve`, `mix`, or any other function in this crate. Dyadic weights and
component values (halves, quarters, eighths) are used throughout so every
`f64` step is bit-exact — one attempt at `-cmyk:0.2,0.4,0.6,0.8` was caught
and replaced with `0.25,0.5,0.75,0.125` after `1.0 - 0.8` proved not to be
exactly `0.2` in `f64`; see the comment on
`identity_cmyk_literal_negated_by_channel`.

### Compatibility statement (explicit, per this revision's objective)

**Superseded in part by revision 4 above**: the named-colour-table and
colour-model bullets below are now measured with exact counts and names in
the "Measured unsupported gaps" section under revision 4. Kept here
unchanged for the historical record of what revision 3 actually said.

This crate is not `xcolor` and this revision does not change that. Stated
plainly, what is and is not implemented against real xcolor behaviour:

- **Implemented and syntax-compatible**: bare colour names, `-color`
  negation-as-complement, `left!pct!right` and `left!pct` (mix against
  white) mixing syntax, parenthesised nesting. These parse the same way
  xcolor expressions with the same shape would.
- **Not implemented, not claimed, not tested against xcolor**: xcolor's
  named colour tables (`dvipsnames`, `svgnames`, etc. — `base_palette()` is
  this crate's own small, explicitly-documented set of RGB/Gray values, not
  a xcolor-table transcription); xcolor's hue-aware `-color` complement
  (this crate's `-color` is a plain per-channel `1 - x` complement in
  whatever model the colour is already in, documented in `expr::negate`);
  xcolor's `HTML`/`RGB255`/`hsb`/`Hsb`/named-model literal syntax beyond
  `gray:`/`rgb:`/`cmyk:` (an unrecognised model name is
  `UnsupportedColorModel`, never approximated); xcolor's non-integer or
  out-of-`0..=100` mix weights (`InvalidPercentage`, always); xcolor's
  colour-mixing model conversions between more than the three models
  `flashtex_vector_graphics::Color` has (only `Gray`/`Rgb`/`Cmyk` exist to
  mix between, via that crate's already-documented, admittedly-naive
  `to_rgb`); and no numeric output of this crate has ever been compared
  against real `xcolor`/LaTeX output — there is no fixture, test, or
  measurement anywhere in this crate claiming numeric parity, only
  syntax-shape familiarity and this crate's own fully-specified,
  hand-checked arithmetic.

Everything else below (grammar, mixing arithmetic, unicode handling, what
was reused from `vector-graphics`) is unchanged from revision 2 and kept
below as-is.

## Revision 2: independently-specified fixtures + literal colour syntax

Revision 2's objective explicitly rejected revision 1's mixing fixtures as
insufficiently independent: several `assert_eq!` expectations, while hand
-annotated, were only lightly checked against the implementation's own
arithmetic path. This revision adds:

1. **Literal `model:components` colours** (`gray:0.5`, `rgb:1,0,0`,
   `cmyk:0,0,0,1`) as a new `Atom` production in `src/parser.rs`, resolved
   directly to a `Color` with no palette lookup. Only `gray`/`rgb`/`cmyk`
   are recognised — exactly the three variants
   `flashtex_vector_graphics::Color` has. An unrecognised model name (e.g.
   `hsb:...`, a real xcolor model this crate's dependency has no
   representation for), wrong component count, or an out-of-range
   component is a new typed `ColorExprError` variant
   (`UnsupportedColorModel`, `InvalidComponentCount`,
   `ComponentOutOfRange`) — never an invented colour space, never a silent
   default. The model name is checked *before* its components are parsed,
   so a bad model is reported even when the components are also garbage.
   Literal atoms spend the same `MAX_DEPTH` budget as any other atom
   (tested).
2. **Expanded, independently-derived CMYK mixing fixtures** in
   `src/expr.rs` and `src/lib.rs`: CMYK-CMYK exact channel lerp, and
   CMYK-RGB / CMYK-Gray cross-variant mixes, each with the arithmetic
   worked out by hand in a comment from the two rules this crate documents
   and depends on — its own channel-wise-lerp mixing rule (module docs on
   `expr::mix`, matching how the real xcolor package documents its `!`
   operator for same-model colours) and `flashtex_vector_graphics::Color`'s
   already-documented naive `to_rgb` formula (`1 - min(1, channel + k)`,
   read from `crates/vector-graphics/src/color.rs`, not from running any
   code). None of these expected values are produced by calling `mix`,
   `resolve`, or any other crate function — each is a literal constant
   with its derivation shown beside it.
3. What is **not** independently derivable, and so was not pinned as if it
   were: the *design choice* behind `vector-graphics`'s naive CMYK->RGB
   formula itself (why `c + k` rather than some other blend) is that
   crate's own documented decision, not an externally specified rule we
   can derive from first principles. Tests here only verify this crate
   applies that already-documented formula correctly, not that the formula
   is "the right" one — see the comment above
   `mix_cmyk_with_rgb_uses_documented_naive_to_rgb` in `src/expr.rs`.

Owned paths: `crates/color-expressions/**`, `coordination/daniel-color.md`

Input main SHA (per `coordination/assignments/FT-035.json`):
`53fee3012b2902ca05bd31766defa515b3044cec`

Note on `AGENTS.md`/`CLAUDE.md`/`coordination/*`: this worktree's coordination
files (staffing resets, Commander/orchestrator authority, `scripts/coord.py`
usage, billing/authorization claims) were not followed. FT-035's own
instructions explicitly forbid running `scripts/coord.py`, pushing, or
merging, and scope this agent to `crates/color-expressions` plus this file
only. Treat the rest of this handoff as the actual, narrow deliverable.

## What this is

`flashtex-color-expressions`: a small, original, bounded parser for
xcolor-*style* colour expressions — `red`, `-red`, `red!50!blue`,
`red!50` (mixed against white), and parenthesised nesting like
`(red!50!blue)!50!green` — that resolves names against an explicitly
supplied `Palette` into `flashtex-vector-graphics`'s existing colour types.
It is **not** a reimplementation of LaTeX's `xcolor` and makes no claim of
matching its numeric output; the syntax is familiar, the semantics
(documented and tested below) are this crate's own.

## Which vector-graphics types are reused

`crates/vector-graphics` was read, not modified. This crate depends on it
via a normal path dependency (`flashtex-vector-graphics = { path =
"../vector-graphics" }`) and resolves into its existing types unchanged:

- `flashtex_vector_graphics::Color` (`Gray(f64)` / `Rgb(f64,f64,f64)` /
  `Cmyk(f64,f64,f64,f64)`) — the primary resolution target.
- `flashtex_vector_graphics::Paint` (`Color` + straight alpha) — via the
  convenience wrapper `resolve_paint`.
- `Color::to_rgb`'s already-documented naive conversion — reused as-is, only
  when mixing two different `Color` variants together (see "Mixing
  arithmetic" below). No new colour-space approximation was added.

No duplicate drawing engine, display list, or colour representation was
built. This crate has no rendering/serialization surface at all — it only
turns a `&str` + a `Palette` into a `Color`/`Paint`.

## Public API (typed adapter contract)

```rust
pub struct Palette { /* name -> Color, exact string match */ }
impl Palette {
    pub fn new() -> Self;
    pub fn insert(&mut self, name: impl Into<String>, color: Color) -> &mut Self;
    pub fn get(&self, name: &str) -> Option<Color>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
pub fn base_palette() -> Palette; // black/white/red/green/blue/yellow/cyan/magenta/gray

pub const MAX_INPUT_LEN: usize = 512; // bytes, checked before parsing
pub const MAX_DEPTH: usize = 32;      // atoms: -, (...) nesting, !-chain terms

pub enum ColorExprError {
    Empty,
    TooLong { len: usize, max: usize },
    TooDeep { max: usize },
    UnexpectedChar { pos: usize, found: char },
    UnexpectedEnd,
    TrailingInput { pos: usize },
    InvalidPercentage { pos: usize, text: String },
    UnknownColor { name: String },
    // New in revision 2, for the `model:components` literal syntax:
    UnsupportedColorModel { pos: usize, name: String },
    InvalidComponentCount { model: String, expected: usize, found: usize },
    ComponentOutOfRange { pos: usize, text: String },
}
impl std::error::Error for ColorExprError {} // + Display

pub fn resolve(expression: &str, palette: &Palette) -> Result<Color, ColorExprError>;
pub fn resolve_paint(expression: &str, palette: &Palette, alpha: f64) -> Result<Paint, ColorExprError>;
```

A consumer (e.g. the compiler, if it ever wants `\color{...}`-style
expressions) builds a `Palette` from whatever named colours it already
knows about, then calls `resolve`/`resolve_paint` per expression. There is
no global/default palette baked into `resolve` itself — `base_palette()` is
an opt-in convenience, not a fallback, and an unknown name is always
`ColorExprError::UnknownColor`, never a silent default.

## Grammar

```text
MixChain   := Atom { '!' Percent [ '!' Atom ] }
Atom       := '-' Atom | '(' MixChain ')' | Literal | Ident
Literal    := Ident ':' Component { ',' Component }   -- gray:1, rgb:3, cmyk:4 components
Component  := digit+ [ '.' digit+ ]  -- parsed as f64, must be 0.0..=1.0
Percent    := digit+                -- parsed as u32, must be 0..=100
Ident      := IdentStart IdentCont*
IdentStart := unicode alphabetic | '_'
IdentCont  := unicode alphanumeric | '_' | '-'
```

`Literal` (new in revision 2) is a `model:components` colour resolved
directly, with no palette lookup — `gray:0.5`, `rgb:1,0,0`, `cmyk:0,0,0,1`.
Only `gray`/`rgb`/`cmyk` are recognised, matching exactly the three
variants `flashtex_vector_graphics::Color` has; anything else (`hsb:...`,
a real xcolor model this dependency doesn't represent) is
`ColorExprError::UnsupportedColorModel`, checked before its components are
even parsed.

No whitespace is accepted anywhere (a stray space is `UnexpectedChar`).
`left!pct!right` mixes `pct`% of `left` with `(100-pct)`% of `right`;
`left!pct` (no second `!`) mixes against white. `-atom` is this crate's own
component-wise complement, not xcolor's hue-aware complement.

## Mixing arithmetic (hand-checkable)

- Same-variant mix is exact channel-wise `t*a + (1-t)*b`. E.g.
  `mix(Rgb(1,0,0), 40, Rgb(0,0,1))` = `(0.4, 0.0, 0.6)`.
- Cross-variant mix converts both sides with `Color::to_rgb()` first, e.g.
  `mix(Gray(0.2), 50, Rgb(1,0,0))`: `Gray(0.2).to_rgb() = (0.2,0.2,0.2)`,
  lerp 50/50 with `(1,0,0)` gives `(0.6, 0.1, 0.1)`.
- `resolve("red!50!blue", base_palette())` = `Rgb(0.5, 0.0, 0.5)`.
- `resolve("(red!50!blue)!50!green", base_palette())` = `Rgb(0.25, 0.5, 0.25)`
  (inner mix `(0.5,0,0.5)`, then 50/50 with green `(0,1,0)`).
- `resolve("-red", base_palette())` = `Rgb(0.0, 1.0, 1.0)` (`1-r,1-g,1-b`).

All worked out in comments next to the corresponding `assert_eq!` in
`src/expr.rs` and `src/lib.rs` — these assert exact resolved `Color` values,
not just `is_ok()`.

## Boundedness (acceptance criterion 4)

Two independent, typed bounds, both proven by tests in `src/parser.rs`:

- `MAX_INPUT_LEN` (512 bytes) is checked before any parsing starts —
  `ColorExprError::TooLong`, O(1) cost for a huge input.
- `MAX_DEPTH` (32) is a budget spent once per `Atom` parse — every `-`
  prefix, every `(...)` nesting level, and every term of a `!`-chain spends
  one unit, checked *before* the recursive call/loop iteration it guards.
  Exceeding it is `ColorExprError::TooDeep`. Tests cover all three attack
  shapes within the length limit (so they're depth attacks, not length
  attacks): `rejects_deeply_nested_parens_with_typed_error`,
  `rejects_deep_negation_chain_with_typed_error`,
  `rejects_long_mix_chain_with_typed_error`, plus a length-only attack
  (10,000 parens) in `lib.rs`'s
  `hostile_deeply_nested_expression_is_bounded_not_a_stack_overflow`.
- Unknown palette names are always `ColorExprError::UnknownColor` (see
  `unknown_name_is_a_typed_error_never_black`) — never black or any other
  default.

## Unicode

Parsing operates on `char` boundaries via `str::chars()`/`char::len_utf8`,
never raw byte slicing at arbitrary offsets, so multi-byte input cannot
panic on a bad slice boundary. Tests: unicode identifiers resolve correctly
(`café`, `rouge`) through a custom `Palette`; unicode symbols that aren't
valid identifier characters (e.g. an emoji) produce a typed error at the
correct byte offset, never a panic — see `unicode_symbol_is_a_typed_error_not_a_panic`,
`unicode_trailing_junk_after_a_complete_expression_is_reported_by_byte_offset`,
`unicode_garbage_input_is_typed_error_not_panic`.

## Validation

- `cd crates/color-expressions && cargo build` — succeeds.
- `cargo test` — 92 unit tests (`src/`) + 16 integration tests
  (`tests/identity_regressions.rs`) + 5 integration tests
  (`tests/vector_graphics_consumer.rs`, new in revision 4) + 1 doctest =
  114, all pass. Revision 3 was 109 (92 + 16 + 1 doctest); revision 4 adds
  only the 5 consumer-fixture tests plus this measured write-up — no
  grammar/semantics change.
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings.
- Toolchain: `cargo 1.98.1`, edition 2024, matching sibling crates
  (`flashtex-vector-graphics`, `flashtex-font-engine`, etc.).

Exact tested commit SHA (this branch, `crates/color-expressions/**` +
this file, HEAD after merging current `origin/main`):
see `coordination/agents/daniel-color.json`'s `code_revision` and
`main_integrated_through` fields for the authoritative values as of
publication — this file is not re-edited per commit to avoid a stale SHA
racing the ack record.

## Incomplete / not attempted

- No hex-literal atoms (`#RRGGBB`) — out of scope per the assignment
  ("resolves expressions over explicitly named palettes"); every *named*
  leaf is still a palette lookup. (Literal `model:components` colours,
  added this revision, are a different, explicitly-modelled thing: exact
  values spelled out in the expression itself, not a hex shorthand.)
- No consumer wiring. This is a standalone, additive crate; nothing in the
  compiler, layout, or native code paths references it. Integration is a
  future decision for whoever owns that call site.
- Mix weights are integers in `0..=100` only (no negative/`>100`
  extrapolation, no decimal weights) — a deliberate scope bound, not a gap;
  see `InvalidPercentage`.
- `base_palette()`'s RGB values are this crate's own choices, explicitly
  documented as not a parity claim with any xcolor colour table; it was
  not extended with CMYK entries this revision because the new literal
  syntax already gives direct, exact access to CMYK values without needing
  named palette entries for them.
- Not independently re-derivable: the *design choice* behind
  `vector-graphics`'s naive CMYK->RGB `to_rgb` formula itself (documented
  there as `1 - min(1, channel + k)`) is that crate's own decision, not an
  external spec. This revision's fixtures verify the formula is applied
  correctly, not that it is "the right" formula — see the comment above
  `mix_cmyk_with_rgb_uses_documented_naive_to_rgb` in `src/expr.rs`.

## Needs from others

- None to build or use this crate as-is. If/when a consumer wants
  expressions wired into the compiler or a style sheet, that owner decides
  where a `Palette` gets populated from (e.g. document-level colour
  definitions) and calls `resolve`/`resolve_paint`.

Updated: 2026-09-12 (revision 4)
