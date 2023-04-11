use super::Drawable;
use super::*;
use crate::style::{Color, ShapeStyle, SizeDesc};
use plotters_backend::{DrawingBackend, DrawingErrorKind};

/**
A common trait for elements that can be interpreted as points: A cross, a circle, a triangle marker...

This is used internally by Plotters and should probably not be included in user code.
See [`ComposedElement`] for more information and examples.
*/
pub trait PointElement<Coord, Size: SizeDesc> {
    /**
    Point creator.

    This is used internally by Plotters and should probably not be included in user code.
    See [`ComposedElement`] for more information and examples.
    */
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self;
}

/**
A cross marker for visualizing data series.

See [`ComposedElement`] for more information and examples.
*/
pub struct Cross<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> Cross<Coord, Size> {
    /**
    Creates a cross marker.

    See [`ComposedElement`] for more information and examples.
    */
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord, Size: SizeDesc> Drawable<Coord> for Cross<Coord, Size> {
    fn draw<CT: CoordTranslate<From = Coord>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        let (x, y) = BackendCoordOnly::map(coord_trans, &self.center, clipping_box);
        let size = self.size.in_pixels(&ps);
        let (x0, y0) = (x - size, y - size);
        let (x1, y1) = (x + size, y + size);
        backend.draw_line((x0, y0), (x1, y1), self.style.into())?;
        backend.draw_line((x0, y1), (x1, y0), self.style.into())
    }
}

/**
A triangle marker for visualizing data series.

See [`ComposedElement`] for more information and examples.
*/
pub struct TriangleMarker<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> TriangleMarker<Coord, Size> {
    /**
    Creates a triangle marker.

    See [`ComposedElement`] for more information and examples.
    */
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord, Size: SizeDesc> Drawable<Coord> for TriangleMarker<Coord, Size> {
    fn draw<CT: CoordTranslate<From = Coord>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        let (x, y) = BackendCoordOnly::map(coord_trans, &self.center, clipping_box);
        let size = self.size.in_pixels(&ps);
        let points: Vec<_> = [-90, -210, -330]
            .iter()
            .map(|deg| f64::from(*deg) * std::f64::consts::PI / 180.0)
            .map(|rad| {
                (
                    (rad.cos() * f64::from(size) + f64::from(x)).ceil() as i32,
                    (rad.sin() * f64::from(size) + f64::from(y)).ceil() as i32,
                )
            })
            .collect();
        backend.fill_polygon(&points[..], self.style.color.to_backend_color().into())
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Cross<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for TriangleMarker<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Circle<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Pixel<Coord> {
    fn make_point(pos: Coord, _: Size, style: ShapeStyle) -> Self {
        Self::new(pos, style)
    }
}
