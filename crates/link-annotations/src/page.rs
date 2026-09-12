//! Exportable page targets: the page and rectangle a PDF exporter needs to
//! build a destination (e.g. a `/GoTo` array) for a resolved internal link.
//!
//! Pure data model: nothing here writes a PDF, looks up a page count, or
//! performs any I/O. It only carries the numbers an exporter would need.

use crate::geometry::Rect;

/// A 0-based page index in the exported document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageIndex(u32);

impl PageIndex {
    pub fn new(index: u32) -> PageIndex {
        PageIndex(index)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Where a resolved internal target lands for a PDF exporter: which page,
/// and the rectangle on it that should be framed when a reader navigates
/// there. This is destination data only — it never resolves against an
/// actual rendered document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageTarget {
    pub page: PageIndex,
    pub rect: Rect,
}

impl PageTarget {
    pub fn new(page: PageIndex, rect: Rect) -> PageTarget {
        PageTarget { page, rect }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn carries_real_page_and_rect_values() {
        let rect = Rect::new(Point::new(72.0, 640.0), 200.0, 40.0).unwrap();
        let target = PageTarget::new(PageIndex::new(3), rect);
        assert_eq!(target.page.value(), 3);
        assert_eq!(target.rect.origin().x(), 72.0);
        assert_eq!(target.rect.max_y(), 680.0);
    }

    #[test]
    fn page_index_zero_based_and_ordered() {
        assert!(PageIndex::new(0) < PageIndex::new(1));
        assert_eq!(PageIndex::new(5).value(), 5);
    }
}
