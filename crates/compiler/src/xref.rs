//! LaTeX counters for cross-references: `\newcounter{name}[within]`,
//! `\numberwithin`, `\refstepcounter` and `\the<name>`.
//!
//! Shared by every numbered construct. Section headings use it today; any
//! other numbered environment (theorems, tables, ...) adopts it the same way:
//! `define` (or `number_within`) the counter once, then call `step` where
//! LaTeX calls `\refstepcounter` and store the returned value as the parser's
//! current `\label` value (`P::current_counter`). Labels themselves stay
//! `Inline::Label` values resolved by `layout::layout_converged`.
//!
//! Counters live in a small `Vec` in definition order, so iteration and
//! therefore output are deterministic.

/// One named counter.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Counter {
    name: String,
    value: u32,
    /// Index of the counter whose step resets this one.
    reset_by: Option<usize>,
    /// `\the<name>` is `\the<parent>.<value>` rather than plain `<value>`.
    prefixed: bool,
}

/// The document's counter table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counters {
    counters: Vec<Counter>,
}

impl Counters {
    /// article.cls: `section`, `subsection` numbered within `section`, and
    /// `subsubsection` numbered within `subsection` (`\thesubsection` is
    /// `\thesection.\arabic{subsection}`).
    pub fn article() -> Self {
        let mut counters = Counters::default();
        counters.define("section", None);
        counters.number_within("subsection", "section");
        counters.number_within("subsubsection", "subsection");
        counters
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.counters
            .iter()
            .position(|counter| counter.name == name)
    }

    /// `\newcounter{name}[within]`: reset by `within`, printed as plain
    /// arabic. Returns false (changing nothing) when `name` already exists or
    /// `within` does not, mirroring LaTeX's errors for both cases.
    pub fn define(&mut self, name: &str, within: Option<&str>) -> bool {
        self.insert(name, within, false)
    }

    /// amsmath `\numberwithin{name}{parent}` semantics for a new counter: reset
    /// by `parent` and printed as `\the<parent>.<value>`.
    pub fn number_within(&mut self, name: &str, parent: &str) -> bool {
        self.insert(name, Some(parent), true)
    }

    /// amsmath `\numberwithin{name}{parent}` for an existing counter: from
    /// now on `name` resets with `parent` and prints as
    /// `\the<parent>.<value>`. Returns false (changing nothing) when either
    /// counter is undefined or `parent` is already numbered within `name`.
    pub fn set_within(&mut self, name: &str, parent: &str) -> bool {
        let (Some(index), Some(parent_index)) = (self.index(name), self.index(parent)) else {
            return false;
        };
        let mut ancestor = Some(parent_index);
        while let Some(at) = ancestor {
            if at == index {
                return false;
            }
            ancestor = self.counters[at].reset_by;
        }
        self.counters[index].reset_by = Some(parent_index);
        self.counters[index].prefixed = true;
        true
    }

    fn insert(&mut self, name: &str, within: Option<&str>, prefixed: bool) -> bool {
        if self.index(name).is_some() {
            return false;
        }
        let reset_by = match within {
            Some(parent) => match self.index(parent) {
                Some(index) => Some(index),
                None => return false,
            },
            None => None,
        };
        self.counters.push(Counter {
            name: name.to_string(),
            value: 0,
            reset_by,
            prefixed: prefixed && reset_by.is_some(),
        });
        true
    }

    /// `\refstepcounter{name}`: increment, reset every counter numbered
    /// within it (transitively, as `\@stpelt` does), and return `\the<name>`.
    pub fn step(&mut self, name: &str) -> Option<String> {
        let index = self.index(name)?;
        self.counters[index].value += 1;
        self.reset_descendants(index);
        self.the(name)
    }

    fn reset_descendants(&mut self, parent: usize) {
        for child in 0..self.counters.len() {
            if self.counters[child].reset_by == Some(parent) {
                self.counters[child].value = 0;
                self.reset_descendants(child);
            }
        }
    }

    /// `\value{name}`.
    pub fn value(&self, name: &str) -> Option<u32> {
        self.index(name).map(|index| self.counters[index].value)
    }

    /// `\the<name>`.
    pub fn the(&self, name: &str) -> Option<String> {
        self.index(name).map(|index| self.format(index))
    }

    fn format(&self, index: usize) -> String {
        let counter = &self.counters[index];
        match counter.reset_by {
            Some(parent) if counter.prefixed => {
                format!("{}.{}", self.format(parent), counter.value)
            }
            _ => counter.value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_sectioning_numbers_and_resets_transitively() {
        let mut counters = Counters::article();
        assert_eq!(counters.step("section").as_deref(), Some("1"));
        assert_eq!(counters.step("subsection").as_deref(), Some("1.1"));
        assert_eq!(counters.step("subsubsection").as_deref(), Some("1.1.1"));
        assert_eq!(counters.step("subsection").as_deref(), Some("1.2"));
        assert_eq!(counters.value("subsubsection"), Some(0));
        counters.step("subsubsection");
        assert_eq!(counters.step("section").as_deref(), Some("2"));
        assert_eq!(counters.value("subsection"), Some(0));
        assert_eq!(counters.value("subsubsection"), Some(0));
        assert_eq!(counters.step("subsubsection").as_deref(), Some("2.0.1"));
    }

    #[test]
    fn newcounter_within_resets_but_prints_plain_arabic() {
        let mut counters = Counters::article();
        assert!(counters.define("theorem", Some("section")));
        assert!(!counters.define("theorem", None), "redefinition is refused");
        assert!(!counters.define("lemma", Some("missing")));
        counters.step("section");
        counters.step("theorem");
        assert_eq!(counters.step("theorem").as_deref(), Some("2"));
        counters.step("section");
        assert_eq!(counters.the("theorem").as_deref(), Some("0"));
        assert_eq!(counters.step("unknown"), None);
    }
}
