# daniel-spelling handoff (FT-039)

Agent / task / branch: `daniel-spelling` / FT-039 "spellcheck" /
`agent/daniel-spelling/spellcheck`

State: ready for integration (standalone additive crate; not yet wired into
any consumer)

Owned paths: `crates/spellcheck/**`, `coordination/daniel-spelling.md`. No
other crate or coordination file was touched.

## What this is

`flashtex-spellcheck` (`crates/spellcheck`): offline, bounded, source-aware
spelling suggestions over caller-supplied text and a caller-supplied
dictionary.

- **Zero network, zero bundled dictionary.** The crate has no dependencies
  at all (`Cargo.toml` has an empty `[dependencies]` table), does no I/O,
  and ships no word list. Callers construct their own `Dictionary`.
- **Exact revisions preserved.** The checker never rewrites or normalizes
  the input text; `check()` only reads `text` and returns byte ranges into
  it plus suggestion strings. There is no "apply correction" API.
- **Math and command exclusion.** Text inside `$...$`, `$$...$$`, `\(...\)`,
  `\[...\]`, and inside a LaTeX command name (`\foo`, `\foo*`) is excluded
  from checking.
- **UTF-8 safe.** All ranges are derived from `char_indices`, so they never
  split a multi-byte character.
- **Bounded suggestion generation**, independent of dictionary size (see
  below).

## Typed contract (public API)

```rust
pub trait Dictionary {
    fn contains(&self, word: &str) -> bool;
}
// Provided impls: HashSet<String>, HashSet<&str>, BTreeSet<String>, BTreeSet<&str>.

pub struct SpellCheckerConfig {
    pub max_edit_distance: usize,               // default 2, hard-clamped to 2
    pub max_suggestions: usize,                  // default 5
    pub max_word_length_for_suggestions: usize,  // default 20 (chars)
}
impl Default for SpellCheckerConfig { .. }

pub struct Misspelling {
    pub word: String,
    pub range: std::ops::Range<usize>, // byte range in the input text, char-boundary safe
    pub suggestions: Vec<String>,       // ranked: distance asc, then lexicographic; capped
}

pub struct SpellChecker { .. }
impl SpellChecker {
    pub const MAX_ALLOWED_EDIT_DISTANCE: usize = 2;
    pub fn new(config: SpellCheckerConfig) -> Self;
    pub fn config(&self) -> &SpellCheckerConfig;
    pub fn check(&self, text: &str, dictionary: &dyn Dictionary) -> Vec<Misspelling>;
}
impl Default for SpellChecker { .. } // SpellChecker::new(SpellCheckerConfig::default())
```

No consumer currently calls this crate, so there is no existing contract to
break. A future integrator should treat `Dictionary` as the seam: build a
`HashSet<String>`/`BTreeSet<String>` from whatever word list the host
application already owns and pass it by reference to `check`.

## Suggestion bound (the acceptance-critical part)

Suggestions are generated from the *misspelled word itself*
(insertion/deletion/substitution/transposition over a fixed 26-letter ASCII
alphabet — a stated, deliberate limitation, not an oversight), then each
candidate is checked against the dictionary with one `contains` call.
**Dictionary size never enters the cost model** — the dictionary is never
iterated or diffed against.

Concrete bounds enforced:
- `max_edit_distance` is silently clamped to `MAX_ALLOWED_EDIT_DISTANCE = 2`
  (raw candidate count grows roughly as `(26 * len)^distance`).
- Words longer than `max_word_length_for_suggestions` (default 20 chars)
  still get reported as misspelled, but suggestion generation is skipped
  entirely for them — this is what actually protects against a pathological
  long "word" in malformed input.
- A hard ceiling `MAX_RAW_CANDIDATES = 50_000` on raw (pre-filter) strings
  examined during the distance-2 fallback pass, as defense in depth on top
  of the length cap.
- `max_suggestions` (default 5) caps the returned list regardless of how
  many dictionary matches were found.

Tested in `long_word_gets_no_suggestions_but_is_still_flagged_and_is_fast`
(200-char non-word, asserts empty suggestions and sub-2-second completion)
and `suggestion_count_never_exceeds_configured_cap_even_with_large_dictionary`
(a word one substitution away from all 26 dictionary entries of a given
shape, plus 5,000 unrelated entries — asserts the returned list is capped at
`max_suggestions` and matches the exact expected top-N alphabetically).

## Math/command exclusion and UTF-8 safety — how it was proven

- `inline_math_is_excluded_but_body_text_is_checked`: the same misspelling
  in body text is flagged; the identical word inside `$...$` is not.
- `display_math_double_dollar_is_excluded`, `paren_and_bracket_math_are_excluded`:
  same proof for `$$...$$`, `\(...\)`, `\[...\]`.
- `command_name_is_excluded_but_same_word_in_body_is_checked`: `\wrold` (the
  command name) is not flagged; the same word later in body text is.
- `starred_command_name_is_excluded`, `escaped_special_characters_do_not_open_math`
  (`\$` does not toggle math mode), `degenerate_double_backslash_command_does_not_panic`.
- UTF-8 safety: `multibyte_word_range_is_char_boundary_safe_and_correct`
  (`café`, asserting the exact 0..5 byte range and `is_char_boundary` at both
  ends) and `multibyte_math_exclusion_boundaries_are_char_safe` (a math span
  containing `ö`, asserting the excluded range is exactly `$föö wrold$` with
  char-boundary-safe start/end). `emoji_and_zero_width_characters_do_not_panic_or_split`
  covers an astral-plane emoji and a zero-width space.
- Malformed-input robustness (no panics, graceful degradation to
  "excluded to end of text"): `unterminated_math_delimiter_excludes_to_end_without_panicking`,
  `trailing_lone_backslash_does_not_panic`, `dollar_at_end_of_input_does_not_panic`,
  `empty_input_produces_no_misspellings`.

Known, deliberate limitations (out of scope for FT-039 as specified):
LaTeX math *environments* (`\begin{equation}...\end{equation}` etc.) are not
recognized as math — only `$...$`, `$$...$$`, `\(...\)`, `\[...\]`. Command
*arguments* (text in `{...}` after a command) are treated as normal prose
and are checked, matching the acceptance text's specific example ("inside a
command name"), not the whole command invocation. The suggestion alphabet is
ASCII a-z only, so non-Latin dictionaries get correct detection/exclusion
but no generated suggestions.

## Validation

```
cd crates/spellcheck
cargo build            # clean
cargo test              # 26 passed, 0 failed (25 unit-style tests + 1 dictionary-impl test), 0 doctests
cargo clippy --all-targets -- -D warnings   # clean, zero warnings
```

rustc/cargo: 1.98.1 (from `rustup`, per assignment setup).

No network access was made or required at any point (empty dependency
tree; `cargo build`/`test`/`clippy` ran entirely from the local toolchain).

## Exact tested commit

SHA: **`<FILLED IN AFTER COMMIT — see below>`**

(Populated by `git rev-parse HEAD` immediately after committing this file
and the crate together; `cargo build`, `cargo test`, and `cargo clippy` were
all re-run clean against that exact commit's working tree before recording
it here.)

## Interface changes / consumer actions

None. This is a new, standalone crate with no existing consumers and no
edits to any other crate, workspace file, or coordination file. No workspace
root `Cargo.toml` was created or modified (none exists in this repository;
each crate builds standalone).

## Needs from others

An integrator who wants to wire this into the compiler/editor pipeline
should open a follow-up task to: (a) decide where the caller-supplied
dictionary is loaded from (file, embedded resource, user settings), and
(b) decide whether command *arguments* should also be excluded for specific
commands (e.g. `\label{...}`, `\cite{...}`, `\ref{...}`) — this crate
intentionally does not make that call, since it's a product decision, not a
spellchecking-primitive decision.

## Next action

None outstanding on this crate for FT-039 as specified. Awaiting review/ack
and, if desired, a separate consumer-integration task.
