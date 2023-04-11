use super::{BackendCoordAndZ, CoordMapper, Drawable};
use crate::{coord::CoordTranslate, drawing::Rect, style::ShapeStyle};
use plotters_backend::{DrawingBackend, DrawingErrorKind};

/**
Represents a cuboid, a six-faced solid.

# Examples

```
use plotters::prelude::*;
let mut backend = SVGBackend::new("cuboid.svg", (300, 200));
let drawing_area = backend.to_drawing_area();
drawing_area.fill(&mut backend, &WHITE).unwrap();
let mut chart_builder = ChartBuilder::on(&drawing_area);
let mut chart_context = chart_builder.margin(20).build_cartesian_3d(&mut backend, 0.0..3.5, 0.0..2.5, 0.0..1.5).unwrap();
chart_context.configure_axes().x_labels(4).y_labels(3).z_labels(2).draw(&mut backend).unwrap();
let cuboid = Cuboid::new([(0.,0.,0.), (3.,2.,1.)], BLUE.mix(0.2), BLUE);
chart_context.draw_series(&mut backend, std::iter::once(cuboid)).unwrap();
```

The result is a semi-transparent cuboid with blue edges:

![](https://cdn.jsdelivr.net/gh/facorread/plotters-doc-data@b6703f7/apidoc/cuboid.svg)
*/
pub struct Cuboid<X, Y, Z> {
    face_style: ShapeStyle,
    edge_style: ShapeStyle,
    vert: [(X, Y, Z); 8],
}

impl<X: Clone, Y: Clone, Z: Clone> Cuboid<X, Y, Z> {
    /**
    Creates a cuboid.

    See [`Cuboid`] for more information and examples.
    */
    #[allow(clippy::redundant_clone)]
    pub fn new<FS: Into<ShapeStyle>, ES: Into<ShapeStyle>>(
        [(x0, y0, z0), (x1, y1, z1)]: [(X, Y, Z); 2],
        face_style: FS,
        edge_style: ES,
    ) -> Self {
        Self {
            face_style: face_style.into(),
            edge_style: edge_style.into(),
            vert: [
                (x0.clone(), y0.clone(), z0.clone()),
                (x0.clone(), y0.clone(), z1.clone()),
                (x0.clone(), y1.clone(), z0.clone()),
                (x0.clone(), y1.clone(), z1.clone()),
                (x1.clone(), y0.clone(), z0.clone()),
                (x1.clone(), y0.clone(), z1.clone()),
                (x1.clone(), y1.clone(), z0.clone()),
                (x1.clone(), y1.clone(), z1.clone()),
            ],
        }
    }
}

impl<X, Y, Z> Drawable<(X, Y, Z)> for Cuboid<X, Y, Z> {
    fn draw<CT: CoordTranslate<From = (X, Y, Z)>, DB: DrawingBackend>(
        &self,
        coord_trans: &CT,
        clipping_box: &Rect,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        let vert: Vec<_> = self
            .vert
            .iter()
            .map(|p| BackendCoordAndZ::map(coord_trans, p, clipping_box))
            .collect();
        let mut polygon = vec![];
        for mask in [1, 2, 4].iter().cloned() {
            let mask_a = if mask == 4 { 1 } else { mask * 2 };
            let mask_b = if mask == 1 { 4 } else { mask / 2 };
            let a = 0;
            let b = a | mask_a;
            let c = a | mask_a | mask_b;
            let d = a | mask_b;
            polygon.push([vert[a], vert[b], vert[c], vert[d]]);
            polygon.push([
                vert[a | mask],
                vert[b | mask],
                vert[c | mask],
                vert[d | mask],
            ]);
        }
        polygon.sort_by_cached_key(|t| std::cmp::Reverse(t[0].1 + t[1].1 + t[2].1 + t[3].1));

        for p in polygon {
            let path: Vec<(i32, i32)> = p.iter().map(|&(coord, _)| coord).chain([p[0].0]).collect();

            backend.fill_polygon(&path[0..4], self.face_style.into())?;
            backend.draw_path(&path[..], self.edge_style.into())?;
        }

        Ok(())
    }
}

#[deprecated(note = "Use Cuboid instead.")]
/// Use [`Cuboid`] instead. This type definition is only for backward-compatibility.
pub type Cubiod<X, Y, Z> = Cuboid<X, Y, Z>;
