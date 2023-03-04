use super::{Drawable, PointCollection};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use std::borrow::Borrow;

trait DynDrawable {
    fn draw_dyn(
        &self,
        points: &mut dyn Iterator<Item = BackendCoord>,
        backend: &mut dyn DrawingBackend,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind>;
}

impl<T: Drawable> DynDrawable for T {
    fn draw_dyn(
        &self,
        points: &mut dyn Iterator<Item = BackendCoord>,
        mut backend: &mut dyn DrawingBackend,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        T::draw(self, points, &mut backend, parent_dim)
    }
}

/// The container for a dynamically dispatched element
pub struct DynElement<'a, Coord>
where
    Coord: Clone,
{
    points: Vec<Coord>,
    drawable: Box<dyn DynDrawable + 'a>,
}

impl<'a, 'b: 'a, Coord: Clone> PointCollection<'a, Coord> for &'a DynElement<'b, Coord> {
    type Point = &'a Coord;
    type IntoIter = &'a Vec<Coord>;
    fn point_iter(self) -> Self::IntoIter {
        &self.points
    }
}

impl<'a, Coord: Clone> Drawable for DynElement<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>, DB: DrawingBackend>(
        &self,
        mut pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        self.drawable.draw_dyn(&mut pos, backend, parent_dim)
    }
}

/// The trait that makes the conversion from the statically dispatched element
/// to the dynamically dispatched element
pub trait IntoDynElement<'a, Coord: Clone>
where
    Self: 'a,
{
    /// Make the conversion
    fn into_dyn(self) -> DynElement<'a, Coord>;
}

impl<'b, T, Coord> IntoDynElement<'b, Coord> for T
where
    T: Drawable + 'b,
    for<'a> &'a T: PointCollection<'a, Coord>,
    Coord: Clone,
{
    fn into_dyn(self) -> DynElement<'b, Coord> {
        DynElement {
            points: self
                .point_iter()
                .into_iter()
                .map(|x| x.borrow().clone())
                .collect(),
            drawable: Box::new(self),
        }
    }
}
