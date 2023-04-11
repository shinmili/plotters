use crate::{coord::CoordTranslate, drawing::Rect};

use super::Drawable;
use plotters_backend::{DrawingBackend, DrawingErrorKind};

pub(crate) trait DynDrawable<Coord> {
    fn draw_dyn(
        &self,
        coord_trans: &dyn CoordTranslate<From = Coord>,
        clipping_box: &Rect,
        backend: &mut dyn DrawingBackend,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind>;
}

impl<Coord, T: Drawable<Coord>> DynDrawable<Coord> for T {
    fn draw_dyn(
        &self,
        coord_trans: &dyn CoordTranslate<From = Coord>,
        clipping_box: &Rect,
        mut backend: &mut dyn DrawingBackend,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        T::draw(self, &coord_trans, clipping_box, &mut backend, parent_dim)
    }
}

/// The container for a dynamically dispatched element
pub struct DynElement<'e, Coord> {
    drawable: Box<dyn DynDrawable<Coord> + 'e>,
}

impl<'e, Coord: Clone> Drawable<Coord> for DynElement<'e, Coord> {
    fn draw<CT: CoordTranslate<From = Coord>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        self.drawable
            .draw_dyn(coord_trans, clipping_box, backend, parent_dim)
    }
}

/// The trait that makes the conversion from the statically dispatched element
/// to the dynamically dispatched element
pub trait IntoDynElement<'e, Coord: Clone> {
    /// Make the conversion
    fn into_dyn(self) -> DynElement<'e, Coord>;
}

impl<'e, T: 'e, Coord> IntoDynElement<'e, Coord> for T
where
    T: Drawable<Coord> + 'e,
    Coord: Clone,
{
    fn into_dyn(self) -> DynElement<'e, Coord> {
        DynElement {
            drawable: Box::new(self),
        }
    }
}
