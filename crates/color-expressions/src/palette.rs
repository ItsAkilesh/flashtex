//! Named palettes: the only source of "leaf" colours an expression can
//! reference. There is no implicit fallback — [`Palette::get`] returns
//! `None` for anything not explicitly inserted, and callers turn that into
//! [`crate::ColorExprError::UnknownColor`], never a default colour.

use flashtex_vector_graphics::Color;
use std::collections::HashMap;

/// An explicit name -> colour mapping used to resolve identifiers in a
/// colour expression.
#[derive(Clone, Debug, Default)]
pub struct Palette {
    entries: HashMap<String, Color>,
}

impl Palette {
    /// An empty palette. Every identifier will be [`crate::ColorExprError::UnknownColor`]
    /// until entries are inserted.
    pub fn new() -> Self {
        Palette {
            entries: HashMap::new(),
        }
    }

    /// Registers (or overwrites) a name. Lookup is by exact string match,
    /// case-sensitive, no normalisation.
    pub fn insert(&mut self, name: impl Into<String>, color: Color) -> &mut Self {
        self.entries.insert(name.into(), color);
        self
    }

    /// Looks up a name. Returns `None` rather than any default colour.
    pub fn get(&self, name: &str) -> Option<Color> {
        self.entries.get(name).copied()
    }

    /// Number of registered names.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the palette has no registered names.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A small set of named base colours, defined here as our own explicit sRGB
/// values. These are chosen for this crate and are **not** a claim of
/// parity with xcolor's `dvipsnames`/`svgnames` tables or any other
/// reference palette.
pub fn base_palette() -> Palette {
    let mut p = Palette::new();
    p.insert("black", Color::Gray(0.0));
    p.insert("white", Color::Gray(1.0));
    p.insert("red", Color::Rgb(1.0, 0.0, 0.0));
    p.insert("green", Color::Rgb(0.0, 1.0, 0.0));
    p.insert("blue", Color::Rgb(0.0, 0.0, 1.0));
    p.insert("yellow", Color::Rgb(1.0, 1.0, 0.0));
    p.insert("cyan", Color::Rgb(0.0, 1.0, 1.0));
    p.insert("magenta", Color::Rgb(1.0, 0.0, 1.0));
    p.insert("gray", Color::Gray(0.5));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_palette_has_no_entries() {
        let p = Palette::new();
        assert!(p.is_empty());
        assert_eq!(p.get("red"), None);
    }

    #[test]
    fn insert_then_get_round_trips() {
        let mut p = Palette::new();
        p.insert("brand", Color::Rgb(0.2, 0.4, 0.6));
        assert_eq!(p.get("brand"), Some(Color::Rgb(0.2, 0.4, 0.6)));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn base_palette_has_expected_reds() {
        let p = base_palette();
        assert_eq!(p.get("red"), Some(Color::Rgb(1.0, 0.0, 0.0)));
        assert_eq!(p.get("black"), Some(Color::Gray(0.0)));
        assert_eq!(p.get("not-a-color"), None);
    }
}
