/*!
  The candlestick element, which showing the high/low/open/close price
*/

use std::cmp::Ordering;

use super::{BackendCoordOnly, CoordMapper, Drawable};
use crate::{coord::CoordTranslate, drawing::Rect, style::ShapeStyle};
use plotters_backend::{DrawingBackend, DrawingErrorKind};

/// The candlestick data point element
pub struct CandleStick<X, Y: PartialOrd> {
    style: ShapeStyle,
    width: u32,
    points: [(X, Y); 4],
}

impl<X: Clone, Y: PartialOrd> CandleStick<X, Y> {
    /// Create a new candlestick element, which requires the Y coordinate can be compared
    ///
    /// - `x`: The x coordinate
    /// - `open`: The open value
    /// - `high`: The high value
    /// - `low`: The low value
    /// - `close`: The close value
    /// - `gain_style`: The style for gain
    /// - `loss_style`: The style for loss
    /// - `width`: The width
    /// - **returns** The newly created candlestick element
    ///
    /// ```rust
    /// use chrono::prelude::*;
    /// use plotters::prelude::*;
    ///
    /// let candlestick = CandleStick::new(Local::now(), 130.0600, 131.3700, 128.8300, 129.1500, &GREEN, &RED, 15);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new<GS: Into<ShapeStyle>, LS: Into<ShapeStyle>>(
        x: X,
        open: Y,
        high: Y,
        low: Y,
        close: Y,
        gain_style: GS,
        loss_style: LS,
        width: u32,
    ) -> Self {
        Self {
            style: match open.partial_cmp(&close) {
                Some(Ordering::Less) => gain_style.into(),
                _ => loss_style.into(),
            },
            width,
            points: [
                (x.clone(), open),
                (x.clone(), high),
                (x.clone(), low),
                (x, close),
            ],
        }
    }
}

impl<'a, X, Y: PartialOrd> Drawable<(X, Y)> for CandleStick<X, Y> {
    fn draw<CT: CoordTranslate<From = (X, Y)>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        let mut points: Vec<_> = self
            .points
            .iter()
            .map(|p| BackendCoordOnly::map(coord_trans, p, clipping_box))
            .collect();
        let fill = self.style.filled;
        if points[0].1 > points[3].1 {
            points.swap(0, 3);
        }
        let (l, r) = (
            self.width as i32 / 2,
            self.width as i32 - self.width as i32 / 2,
        );

        backend.draw_line(points[0], points[1], self.style.into())?;
        backend.draw_line(points[2], points[3], self.style.into())?;

        points[0].0 -= l;
        points[3].0 += r;

        backend.draw_rect(points[0], points[3], self.style.into(), fill)
    }
}
