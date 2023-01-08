use super::*;
use plotters_backend::DrawingBackend;
use std::borrow::Borrow;
use std::iter::{once, Once};
use std::ops::Add;

/**
An empty composable element. This is the starting point of a composed element.

# Example

```
use plotters::prelude::*;
let data = [(1.0, 3.3), (2., 2.1), (3., 1.5), (4., 1.9), (5., 1.0)];
let drawing_area = SVGBackend::new("composable.svg", (300, 200)).into_drawing_area();
drawing_area.fill(&WHITE).unwrap();
let mut chart_builder = ChartBuilder::on(&drawing_area);
chart_builder.margin(7).set_left_and_bottom_label_area_size(20);
let mut chart_context = chart_builder.build_cartesian_2d(0.0..5.5, 0.0..5.5).unwrap();
chart_context.configure_mesh().draw().unwrap();
chart_context.draw_series(data.map(|(x, y)| {
    EmptyElement::at((x, y)) // Use the guest coordinate system with EmptyElement
    + Circle::new((0, 0), 10, BLUE) // Use backend coordinates with the rest
    + Cross::new((4, 4), 3, RED)
    + Pixel::new((4, -4), RED)
    + TriangleMarker::new((-4, -4), 4, RED)
})).unwrap();
```

The result is a data series where each point consists of a circle, a cross, a pixel, and a triangle:

![](https://cdn.jsdelivr.net/gh/facorread/plotters-doc-data@06d370f/apidoc/composable.svg)

*/
pub struct EmptyElement<Coord> {
    coord: Coord,
}

impl<Coord> EmptyElement<Coord> {
    /**
    An empty composable element. This is the starting point of a composed element.

    See [`EmptyElement`] for more information and examples.
    */
    pub fn at(coord: Coord) -> Self {
        Self { coord }
    }
}

impl<Coord, Other> Add<Other> for EmptyElement<Coord>
where
    Other: Drawable,
    for<'a> &'a Other: PointCollection<'a, BackendCoord>,
{
    type Output = BoxedElement<Coord, Other>;
    fn add(self, other: Other) -> Self::Output {
        BoxedElement {
            offset: self.coord,
            inner: other,
        }
    }
}

impl<'a, Coord> PointCollection<'a, Coord> for &'a EmptyElement<Coord> {
    type Point = &'a Coord;
    type IntoIter = Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.coord)
    }
}

impl<Coord> Drawable for EmptyElement<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        _pos: I,
        _backend: &mut dyn DrawingBackend,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        Ok(())
    }
}

/**
A container for one drawable element, used for composition.

This is used internally by Plotters and should probably not be included in user code.
See [`EmptyElement`] for more information and examples.
*/
pub struct BoxedElement<Coord, A: Drawable> {
    inner: A,
    offset: Coord,
}

impl<'b, Coord, A: Drawable> PointCollection<'b, Coord> for &'b BoxedElement<Coord, A> {
    type Point = &'b Coord;
    type IntoIter = Once<&'b Coord>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.offset)
    }
}

impl<Coord, A> Drawable for BoxedElement<Coord, A>
where
    for<'a> &'a A: PointCollection<'a, BackendCoord>,
    A: Drawable,
{
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut dyn DrawingBackend,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        if let Some((x0, y0)) = pos.next() {
            self.inner.draw(
                self.inner.point_iter().into_iter().map(|p| {
                    let p = p.borrow();
                    (p.0 + x0, p.1 + y0)
                }),
                backend,
                ps,
            )?;
        }
        Ok(())
    }
}

impl<Coord, My, Yours> Add<Yours> for BoxedElement<Coord, My>
where
    My: Drawable,
    for<'a> &'a My: PointCollection<'a, BackendCoord>,
    Yours: Drawable,
    for<'a> &'a Yours: PointCollection<'a, BackendCoord>,
{
    type Output = ComposedElement<Coord, My, Yours>;
    fn add(self, yours: Yours) -> Self::Output {
        ComposedElement {
            offset: self.offset,
            first: self.inner,
            second: yours,
        }
    }
}

/**
A container for two drawable elements, used for composition.

This is used internally by Plotters and should probably not be included in user code.
See [`EmptyElement`] for more information and examples.
*/
pub struct ComposedElement<Coord, A, B>
where
    A: Drawable,
    B: Drawable,
{
    first: A,
    second: B,
    offset: Coord,
}

impl<'b, Coord, A, B> PointCollection<'b, Coord> for &'b ComposedElement<Coord, A, B>
where
    A: Drawable,
    B: Drawable,
{
    type Point = &'b Coord;
    type IntoIter = Once<&'b Coord>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.offset)
    }
}

impl<Coord, A, B> Drawable for ComposedElement<Coord, A, B>
where
    for<'a> &'a A: PointCollection<'a, BackendCoord>,
    for<'b> &'b B: PointCollection<'b, BackendCoord>,
    A: Drawable,
    B: Drawable,
{
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut dyn DrawingBackend,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        if let Some((x0, y0)) = pos.next() {
            self.first.draw(
                self.first.point_iter().into_iter().map(|p| {
                    let p = p.borrow();
                    (p.0 + x0, p.1 + y0)
                }),
                backend,
                ps,
            )?;
            self.second.draw(
                self.second.point_iter().into_iter().map(|p| {
                    let p = p.borrow();
                    (p.0 + x0, p.1 + y0)
                }),
                backend,
                ps,
            )?;
        }
        Ok(())
    }
}

impl<Coord, A, B, C> Add<C> for ComposedElement<Coord, A, B>
where
    A: Drawable,
    for<'a> &'a A: PointCollection<'a, BackendCoord>,
    B: Drawable,
    for<'a> &'a B: PointCollection<'a, BackendCoord>,
    C: Drawable,
    for<'a> &'a C: PointCollection<'a, BackendCoord>,
{
    type Output = ComposedElement<Coord, A, ComposedElement<BackendCoord, B, C>>;
    fn add(self, rhs: C) -> Self::Output {
        ComposedElement {
            offset: self.offset,
            first: self.first,
            second: ComposedElement {
                offset: (0, 0),
                first: self.second,
                second: rhs,
            },
        }
    }
}
