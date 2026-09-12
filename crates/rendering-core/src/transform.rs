//! Exact consumer coordinates. Axis-aligned zoom/translation/y-flip use rational
//! arithmetic; no float rounding or platform baseline measurement enters hits.
use crate::{
    hit_test::{Hit, PageIndex, Point},
    *,
};
use std::cmp::Ordering;
const MAX_DENOMINATOR: u64 = 1_000_000;
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RationalTick {
    numerator: i128,
    denominator: u64,
}
impl RationalTick {
    pub fn new(numerator: i128, denominator: u64) -> Result<Self> {
        require(denominator > 0, "zero rational denominator")?;
        let divisor = gcd(
            (numerator.unsigned_abs() % u128::from(denominator)) as u64,
            denominator,
        );
        let denominator = denominator / divisor;
        let numerator = numerator / i128::from(divisor);
        require(
            denominator <= MAX_DENOMINATOR,
            "rational precision budget exceeded",
        )?;
        require(
            numerator.unsigned_abs() <= MAX_EXACT_INTEGER as u128 * u128::from(denominator),
            "rational coordinate out of range",
        )?;
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub fn from_tick(tick: Tick) -> Result<Self> {
        tick.validate()?;
        Self::new(i128::from(tick.0), 1)
    }
    pub fn numerator(self) -> i128 {
        self.numerator
    }
    pub fn denominator(self) -> u64 {
        self.denominator
    }
    /// Floor preserves membership against integer half-open rectangle boundaries.
    /// It is not used to select nearest carets or to paint transformed geometry.
    pub fn floor(self) -> Tick {
        Tick(self.numerator.div_euclid(i128::from(self.denominator)) as i64)
    }
    fn compare_integer(self, value: i64) -> Ordering {
        self.numerator
            .cmp(&(i128::from(value) * i128::from(self.denominator)))
    }
    fn compare(self, other: Self) -> Ordering {
        (self.numerator * i128::from(other.denominator))
            .cmp(&(other.numerator * i128::from(self.denominator)))
    }
    fn offset(self, delta: Tick) -> Result<Self> {
        delta.validate()?;
        Self::new(
            self.numerator + i128::from(delta.0) * i128::from(self.denominator),
            self.denominator,
        )
    }
    fn scale(self, numerator: i64, denominator: u64) -> Result<Self> {
        let n = self
            .numerator
            .checked_mul(i128::from(numerator))
            .ok_or_else(|| ValidationError("rational numerator overflow".into()))?;
        let d = self
            .denominator
            .checked_mul(denominator)
            .ok_or_else(|| ValidationError("rational denominator overflow".into()))?;
        Self::new(n, d)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactPoint {
    pub x: RationalTick,
    pub y: RationalTick,
}
impl ExactPoint {
    pub fn from_point(point: Point) -> Result<Self> {
        Ok(Self {
            x: RationalTick::from_tick(point.x)?,
            y: RationalTick::from_tick(point.y)?,
        })
    }
    pub fn floor(self) -> Point {
        Point {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
    /// Distance numerator under a common fixed denominator for this query. The
    /// checked i128 budget rejects extreme precision instead of approximating.
    pub(crate) fn caret_distance(self, caret: &Caret) -> Result<i128> {
        let common =
            self.x.denominator / gcd(self.x.denominator, self.y.denominator) * self.y.denominator;
        let dx = self.x.numerator * i128::from(common / self.x.denominator)
            - i128::from(caret.x.0) * i128::from(common);
        let bottom = caret.top.0 + caret.height.0;
        let y_edge = if self.y.compare_integer(caret.top.0) == Ordering::Less {
            Some(caret.top.0)
        } else if self.y.compare_integer(bottom) == Ordering::Greater {
            Some(bottom)
        } else {
            None
        };
        let dy = y_edge.map_or(0, |edge| {
            self.y.numerator * i128::from(common / self.y.denominator)
                - i128::from(edge) * i128::from(common)
        });
        dx.checked_mul(dx)
            .and_then(|x| dy.checked_mul(dy).and_then(|y| x.checked_add(y)))
            .ok_or_else(|| ValidationError("exact caret distance overflow".into()))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YAxis {
    Down,
    Up,
}
#[derive(Debug, Clone, Copy)]
pub struct ViewportTransform {
    scale_numerator: u32,
    scale_denominator: u32,
    origin: Point,
    y_axis: YAxis,
}
impl ViewportTransform {
    pub fn new(
        scale_numerator: u32,
        scale_denominator: u32,
        origin: Point,
        y_axis: YAxis,
    ) -> Result<Self> {
        require(
            (1..=1_000_000).contains(&scale_numerator)
                && (1..=1_000_000).contains(&scale_denominator),
            "invalid transform scale",
        )?;
        origin.x.validate()?;
        origin.y.validate()?;
        Ok(Self {
            scale_numerator,
            scale_denominator,
            origin,
            y_axis,
        })
    }
    pub fn forward(self, point: ExactPoint) -> Result<ExactPoint> {
        let n = i64::from(self.scale_numerator);
        let d = u64::from(self.scale_denominator);
        Ok(ExactPoint {
            x: point.x.scale(n, d)?.offset(self.origin.x)?,
            y: point
                .y
                .scale(if self.y_axis == YAxis::Up { -n } else { n }, d)?
                .offset(self.origin.y)?,
        })
    }
    pub fn inverse(self, point: ExactPoint) -> Result<ExactPoint> {
        let n = i64::from(self.scale_denominator);
        let d = u64::from(self.scale_numerator);
        Ok(ExactPoint {
            x: point.x.offset(Tick(-self.origin.x.0))?.scale(n, d)?,
            y: point
                .y
                .offset(Tick(-self.origin.y.0))?
                .scale(if self.y_axis == YAxis::Up { -n } else { n }, d)?,
        })
    }
    pub fn hit_test(
        self,
        index: &PageIndex,
        project: &str,
        revision: u64,
        page: u32,
        view_point: ExactPoint,
    ) -> Result<Option<Hit>> {
        index.hit_test_exact(project, revision, page, self.inverse(view_point)?)
    }
    /// Intersect canonical source geometry before transformation. Empty clips
    /// stay empty; output edges retain exact rational values and y-axis order.
    pub fn visible_bounds(self, rect: &HitRect, clip: &HitRect) -> Result<Option<ExactRect>> {
        rect.validate()?;
        clip.validate()?;
        let left = rect.x.0.max(clip.x.0);
        let top = rect.top.0.max(clip.top.0);
        let right = (rect.x.0 + rect.width.0).min(clip.x.0 + clip.width.0);
        let bottom = (rect.top.0 + rect.height.0).min(clip.top.0 + clip.height.0);
        if left >= right || top >= bottom {
            return Ok(None);
        }
        let a = self.forward(ExactPoint::from_point(Point {
            x: Tick(left),
            y: Tick(top),
        })?)?;
        let b = self.forward(ExactPoint::from_point(Point {
            x: Tick(right),
            y: Tick(bottom),
        })?)?;
        let (top, bottom) = if a.y.compare(b.y) == Ordering::Greater {
            (b.y, a.y)
        } else {
            (a.y, b.y)
        };
        Ok(Some(ExactRect {
            left: a.x,
            right: b.x,
            top,
            bottom,
            top_inclusive: self.y_axis == YAxis::Down,
            bottom_inclusive: self.y_axis == YAxis::Up,
        }))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactRect {
    pub left: RationalTick,
    pub top: RationalTick,
    pub right: RationalTick,
    pub bottom: RationalTick,
    pub top_inclusive: bool,
    pub bottom_inclusive: bool,
}
impl ExactRect {
    pub fn contains(self, point: ExactPoint) -> bool {
        point.x.compare(self.left) != Ordering::Less
            && point.x.compare(self.right) == Ordering::Less
            && (point.y.compare(self.top) == Ordering::Greater
                || (self.top_inclusive && point.y.compare(self.top) == Ordering::Equal))
            && (point.y.compare(self.bottom) == Ordering::Less
                || (self.bottom_inclusive && point.y.compare(self.bottom) == Ordering::Equal))
    }
}
