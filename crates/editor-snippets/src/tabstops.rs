use crate::expand::Expansion;

/// Guarded navigation across a snippet's tab stops.
///
/// Navigating past the last stop, navigating before the first, or
/// navigating a snippet with no placeholders at all is well-defined:
/// [`Self::next`] and [`Self::prev`] simply hold position and return
/// `None` to signal "no further movement happened," rather than
/// panicking, wrapping around, or going out of bounds.
#[derive(Debug, Clone)]
pub struct TabStops {
    order: Vec<u32>,
    /// Index into `order`; `None` means "before the first stop."
    position: Option<usize>,
}

impl TabStops {
    /// Builds the navigation order for `expansion` (see
    /// [`Expansion::tab_order`]). Starts positioned before the first stop.
    pub fn new(expansion: &Expansion) -> Self {
        TabStops {
            order: expansion.tab_order(),
            position: None,
        }
    }

    /// True if there are no placeholders to navigate.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// The currently selected placeholder index, or `None` before the
    /// first call to [`Self::next`], or if there are no placeholders.
    pub fn current(&self) -> Option<u32> {
        self.position.map(|p| self.order[p])
    }

    /// Moves to the next tab stop and returns its index.
    ///
    /// Returns `None`, and leaves the position unchanged, if there are no
    /// placeholders at all, or if already at the last stop - it does not
    /// wrap back to the first, and it never panics.
    pub fn advance(&mut self) -> Option<u32> {
        if self.order.is_empty() {
            return None;
        }
        let last = self.order.len() - 1;
        match self.position {
            None => {
                self.position = Some(0);
                Some(self.order[0])
            }
            Some(p) if p >= last => None,
            Some(p) => {
                self.position = Some(p + 1);
                Some(self.order[p + 1])
            }
        }
    }

    /// Moves to the previous tab stop and returns its index.
    ///
    /// Returns `None`, and leaves the position unchanged, if there are no
    /// placeholders at all, or if already at (or before) the first stop.
    pub fn retreat(&mut self) -> Option<u32> {
        if self.order.is_empty() {
            return None;
        }
        match self.position {
            None | Some(0) => None,
            Some(p) => {
                self.position = Some(p - 1);
                Some(self.order[p - 1])
            }
        }
    }
}
