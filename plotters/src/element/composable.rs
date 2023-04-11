use crate::coord::Shift;

use super::*;
use plotters_backend::DrawingBackend;
use std::ops::Add;

/**
An empty composable element. This is the starting point of a composed element.

# Example

```
use plotters::prelude::*;
let data = [(1.0, 3.3), (2., 2.1), (3., 1.5), (4., 1.9), (5., 1.0)];
let mut backend = SVGBackend::new("composable.svg", (300, 200));
let drawing_area = backend.to_drawing_area();
drawing_area.fill(&mut backend, &WHITE).unwrap();
let mut chart_builder = ChartBuilder::on(&drawing_area);
chart_builder.margin(7).set_left_and_bottom_label_area_size(20);
let mut chart_context = chart_builder.build_cartesian_2d(&mut backend, 0.0..5.5, 0.0..5.5).unwrap();
chart_context.configure_mesh().draw(&mut backend).unwrap();
chart_context
    .draw_series(
        &mut backend,
        data.map(|(x, y)| {
            ComposedElement::at((x, y)) // Use the guest coordinate system with ComposedElement
                + Circle::new((0, 0), 10, BLUE) // Use backend coordinates with the rest
                + Cross::new((4, 4), 3, RED)
                + Pixel::new((4, -4), RED)
                + TriangleMarker::new((-4, -4), 4, RED)
        }),
    )
    .unwrap();
```

The result is a data series where each point consists of a circle, a cross, a pixel, and a triangle:

![](https://cdn.jsdelivr.net/gh/facorread/plotters-doc-data@06d370f/apidoc/composable.svg)

*/

/**
A container for two drawable elements, used for composition.

This is used internally by Plotters and should probably not be included in user code.
See [`ComposedElement`] for more information and examples.
*/
pub struct ComposedElement<'e, Coord> {
    elements: Vec<Box<dyn DynDrawable<BackendCoord> + 'e>>,
    offset: Coord,
}

impl<'e, Coord> ComposedElement<'e, Coord> {
    /**
    An empty composable element. This is the starting point of a composed element.

    See [`ComposedElement`] for more information and examples.
    */
    pub fn at(coord: Coord) -> Self {
        Self {
            elements: vec![],
            offset: coord,
        }
    }
}

impl<'e, Coord> Drawable<Coord> for ComposedElement<'e, Coord> {
    fn draw<CT: CoordTranslate<From = Coord>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        let offset = BackendCoordOnly::map(coord_trans, &self.offset, clipping_box);
        for element in &self.elements {
            element.draw_dyn(&Shift(offset), clipping_box, backend, ps)?;
        }
        Ok(())
    }
}

impl<'e, Coord, E: 'e> Add<E> for ComposedElement<'e, Coord>
where
    E: Drawable<BackendCoord> + 'e,
{
    type Output = Self;

    fn add(mut self, rhs: E) -> Self::Output {
        self.elements.push(Box::new(rhs));
        self
    }
}
