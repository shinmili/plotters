use crate::coord::cartesian::{Cartesian2d, MeshLine};
use crate::coord::ranged1d::{KeyPointHint, Ranged};
use crate::coord::{CoordTranslate, Shift};
use crate::element::{CoordMapper, Drawable, PointCollection};
use crate::style::text_anchor::{HPos, Pos, VPos};
use crate::style::{Color, SizeDesc, TextStyle};

/// The abstraction of a drawing area
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use std::borrow::Borrow;
use std::error::Error;
use std::iter::{once, repeat};
use std::ops::Range;

/// The representation of the rectangle in backend canvas
#[derive(Clone, Debug)]
pub struct Rect {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

impl Rect {
    /// Split the rectangle into a few smaller rectangles
    fn split<'a, BPI: IntoIterator<Item = &'a i32> + 'a>(
        &'a self,
        break_points: BPI,
        vertical: bool,
    ) -> impl Iterator<Item = Rect> + 'a {
        let (mut x0, mut y0) = (self.x0, self.y0);
        let (full_x, full_y) = (self.x1, self.y1);
        break_points
            .into_iter()
            .chain(once(if vertical { &self.y1 } else { &self.x1 }))
            .map(move |&p| {
                let x1 = if vertical { full_x } else { p };
                let y1 = if vertical { p } else { full_y };
                let ret = Rect { x0, y0, x1, y1 };

                if vertical {
                    y0 = y1
                } else {
                    x0 = x1;
                }

                ret
            })
    }

    /// Evenly split the rectangle to a row * col mesh
    fn split_evenly(&self, (row, col): (usize, usize)) -> impl Iterator<Item = Rect> + '_ {
        fn compute_evenly_split(from: i32, to: i32, n: usize, idx: usize) -> i32 {
            let size = (to - from) as usize;
            from + idx as i32 * (size / n) as i32 + idx.min(size % n) as i32
        }
        (0..row)
            .flat_map(move |x| repeat(x).zip(0..col))
            .map(move |(ri, ci)| Self {
                y0: compute_evenly_split(self.y0, self.y1, row, ri),
                y1: compute_evenly_split(self.y0, self.y1, row, ri + 1),
                x0: compute_evenly_split(self.x0, self.x1, col, ci),
                x1: compute_evenly_split(self.x0, self.x1, col, ci + 1),
            })
    }

    /// Evenly the rectangle into a grid with arbitrary breaks; return a rect iterator.
    fn split_grid(
        &self,
        x_breaks: impl Iterator<Item = i32>,
        y_breaks: impl Iterator<Item = i32>,
    ) -> impl Iterator<Item = Rect> {
        let mut xs = vec![self.x0, self.x1];
        let mut ys = vec![self.y0, self.y1];
        xs.extend(x_breaks.map(|v| v + self.x0));
        ys.extend(y_breaks.map(|v| v + self.y0));

        xs.sort_unstable();
        ys.sort_unstable();

        let xsegs: Vec<_> = xs
            .iter()
            .zip(xs.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();

        // Justify: this is actually needed. Because we need to return a iterator that have
        // static life time, thus we need to copy the value to a buffer and then turn the buffer
        // into a iterator.
        #[allow(clippy::needless_collect)]
        let ysegs: Vec<_> = ys
            .iter()
            .zip(ys.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();

        ysegs.into_iter().flat_map(move |(y0, y1)| {
            xsegs
                .clone()
                .into_iter()
                .map(move |(x0, x1)| Self { x0, y0, x1, y1 })
        })
    }

    /// Make the coordinate in the range of the rectangle
    pub fn truncate(&self, p: (i32, i32)) -> (i32, i32) {
        (p.0.min(self.x1).max(self.x0), p.1.min(self.y1).max(self.y0))
    }
}

/// The abstraction of a drawing area. Plotters uses drawing area as the fundamental abstraction for the
/// high level drawing API. The major functionality provided by the drawing area is
///     1. Layout specification - Split the parent drawing area into sub-drawing-areas
///     2. Coordinate Translation - Allows guest coordinate system attached and used for drawing.
///     3. Element based drawing - drawing area provides the environment the element can be drawn onto it.
pub struct DrawingArea<CT> {
    rect: Rect,
    coord: CT,
}

impl<CT: Clone> Clone for DrawingArea<CT> {
    fn clone(&self) -> Self {
        Self {
            rect: self.rect.clone(),
            coord: self.coord.clone(),
        }
    }
}

/// The error description of any drawing area API
#[derive(Debug)]
pub enum DrawingAreaError {
    /// The error is due to drawing backend failure
    BackendError(DrawingErrorKind),
    /// We are not able to get the mutable reference of the backend,
    /// which indicates the drawing backend is current used by other
    /// drawing operation
    SharingError,
    /// The error caused by invalid layout
    LayoutError,
}

impl std::fmt::Display for DrawingAreaError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DrawingAreaError::BackendError(e) => write!(fmt, "backend error: {}", e),
            DrawingAreaError::SharingError => {
                write!(fmt, "Multiple backend operation in progress")
            }
            DrawingAreaError::LayoutError => write!(fmt, "Bad layout"),
        }
    }
}

impl Error for DrawingAreaError {}

/// A type which can be converted into a root drawing area
pub trait ToDrawingArea: DrawingBackend + Sized {
    /// Convert the type into a root drawing area
    fn to_drawing_area(&self) -> DrawingArea<Shift>;
}

impl<T: DrawingBackend> ToDrawingArea for T {
    fn to_drawing_area(&self) -> DrawingArea<Shift> {
        DrawingArea::for_drawing_backend(self)
    }
}

impl<X: Ranged, Y: Ranged> DrawingArea<Cartesian2d<X, Y>> {
    /// Draw the mesh on a area
    pub fn draw_mesh<DB: DrawingBackend, DrawFunc, YH: KeyPointHint, XH: KeyPointHint>(
        &self,
        backend: &mut DB,
        mut draw_func: DrawFunc,
        y_count_max: YH,
        x_count_max: XH,
    ) -> Result<(), DrawingAreaError>
    where
        DrawFunc: FnMut(&mut dyn DrawingBackend, MeshLine<X, Y>) -> Result<(), DrawingErrorKind>,
    {
        self.backend_ops(backend, move |b| {
            self.coord
                .draw_mesh(y_count_max, x_count_max, |line| draw_func(b, line))
        })
    }

    /// Get the range of X of the guest coordinate for current drawing area
    pub fn get_x_range(&self) -> Range<X::ValueType> {
        self.coord.get_x_range()
    }

    /// Get the range of Y of the guest coordinate for current drawing area
    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        self.coord.get_y_range()
    }

    /// Get the range of X of the backend coordinate for current drawing area
    pub fn get_x_axis_pixel_range(&self) -> Range<i32> {
        self.coord.get_x_axis_pixel_range()
    }

    /// Get the range of Y of the backend coordinate for current drawing area
    pub fn get_y_axis_pixel_range(&self) -> Range<i32> {
        self.coord.get_y_axis_pixel_range()
    }
}

impl<CT> DrawingArea<CT> {
    /// Get the left upper conner of this area in the drawing backend
    pub fn get_base_pixel(&self) -> BackendCoord {
        (self.rect.x0, self.rect.y0)
    }

    /// Strip the applied coordinate specification and returns a shift-based drawing area
    pub fn strip_coord_spec(&self) -> DrawingArea<Shift> {
        DrawingArea {
            rect: self.rect.clone(),
            coord: Shift((self.rect.x0, self.rect.y0)),
        }
    }

    /// Strip the applied coordinate specification and returns a drawing area
    pub fn use_screen_coord(&self) -> DrawingArea<Shift> {
        DrawingArea {
            rect: self.rect.clone(),
            coord: Shift((0, 0)),
        }
    }

    /// Get the area dimension in pixel
    pub fn dim_in_pixel(&self) -> (u32, u32) {
        (
            (self.rect.x1 - self.rect.x0) as u32,
            (self.rect.y1 - self.rect.y0) as u32,
        )
    }

    /// Compute the relative size based on the drawing area's height
    pub fn relative_to_height(&self, p: f64) -> f64 {
        f64::from((self.rect.y1 - self.rect.y0).max(0)) * (p.min(1.0).max(0.0))
    }

    /// Compute the relative size based on the drawing area's width
    pub fn relative_to_width(&self, p: f64) -> f64 {
        f64::from((self.rect.x1 - self.rect.x0).max(0)) * (p.min(1.0).max(0.0))
    }

    /// Get the pixel range of this area
    pub fn get_pixel_range(&self) -> (Range<i32>, Range<i32>) {
        (self.rect.x0..self.rect.x1, self.rect.y0..self.rect.y1)
    }

    /// Perform operation on the drawing backend
    fn backend_ops<R, DB: DrawingBackend, O: FnOnce(&mut DB) -> Result<R, DrawingErrorKind>>(
        &self,
        backend: &mut DB,
        ops: O,
    ) -> Result<R, DrawingAreaError> {
        backend
            .ensure_prepared()
            .map_err(DrawingAreaError::BackendError)?;
        ops(backend).map_err(DrawingAreaError::BackendError)
    }

    /// Fill the entire drawing area with a color
    pub fn fill<DB: DrawingBackend, ColorType: Color>(
        &self,
        backend: &mut DB,
        color: &ColorType,
    ) -> Result<(), DrawingAreaError> {
        self.backend_ops(backend, |backend| {
            backend.draw_rect(
                (self.rect.x0, self.rect.y0),
                (self.rect.x1, self.rect.y1),
                color.to_backend_color().into(),
                true,
            )
        })
    }

    /// Present all the pending changes to the backend
    pub fn present<DB: DrawingBackend>(&self, backend: &mut DB) -> Result<(), DrawingAreaError> {
        self.backend_ops(backend, |b| b.present())
    }

    /// Estimate the dimension of the text if drawn on this drawing area.
    /// We can't get this directly from the font, since the drawing backend may or may not
    /// follows the font configuration. In terminal, the font family will be dropped.
    /// So the size of the text is drawing area related.
    ///
    /// - `backend`: The mutable reference to the backend used to estimate
    /// - `text`: The text we want to estimate
    /// - `font`: The font spec in which we want to draw the text
    /// - **return**: The size of the text if drawn on this area
    pub fn estimate_text_size<DB: DrawingBackend>(
        &self,
        backend: &mut DB,
        text: &str,
        style: &TextStyle,
    ) -> Result<(u32, u32), DrawingAreaError> {
        self.backend_ops(backend, move |b| {
            b.estimate_text_size(text, style.clone().into())
        })
    }
}

impl<CT: CoordTranslate> DrawingArea<CT> {
    /// Draw a single pixel
    pub fn draw_pixel<DB: DrawingBackend, ColorType: Color>(
        &self,
        backend: &mut DB,
        pos: CT::From,
        color: &ColorType,
    ) -> Result<(), DrawingAreaError> {
        let pos = self.coord.translate(&pos);
        self.backend_ops(backend, |b| b.draw_pixel(pos, color.to_backend_color()))
    }

    /// Draw an high-level element
    pub fn draw<'e, DB, E, B>(
        &self,
        backend: &mut DB,
        element: &'e E,
    ) -> Result<(), DrawingAreaError>
    where
        DB: DrawingBackend,
        B: CoordMapper,
        &'e E: PointCollection<'e, CT::From, B>,
        E: Drawable<B>,
    {
        let backend_coords = element.point_iter().into_iter().map(|p| {
            let b = p.borrow();
            B::map(&self.coord, b, &self.rect)
        });
        self.backend_ops(backend, move |b| {
            element.draw(backend_coords, b, self.dim_in_pixel())
        })
    }

    /// Map coordinate to the backend coordinate
    pub fn map_coordinate(&self, coord: &CT::From) -> BackendCoord {
        self.coord.translate(coord)
    }
}

impl DrawingArea<Shift> {
    fn for_drawing_backend<DB: DrawingBackend>(backend: &DB) -> Self {
        let (x1, y1) = backend.get_size();
        Self {
            rect: Rect {
                x0: 0,
                y0: 0,
                x1: x1 as i32,
                y1: y1 as i32,
            },
            coord: Shift((0, 0)),
        }
    }

    /// Shrink the region, note all the locations are in guest coordinate
    pub fn shrink<A: SizeDesc, B: SizeDesc, C: SizeDesc, D: SizeDesc>(
        mut self,
        left_upper: (A, B),
        dimension: (C, D),
    ) -> DrawingArea<Shift> {
        let left_upper = (left_upper.0.in_pixels(&self), left_upper.1.in_pixels(&self));
        let dimension = (dimension.0.in_pixels(&self), dimension.1.in_pixels(&self));
        self.rect.x0 = self.rect.x1.min(self.rect.x0 + left_upper.0);
        self.rect.y0 = self.rect.y1.min(self.rect.y0 + left_upper.1);

        self.rect.x1 = self.rect.x0.max(self.rect.x0 + dimension.0);
        self.rect.y1 = self.rect.y0.max(self.rect.y0 + dimension.1);

        self.coord = Shift((self.rect.x0, self.rect.y0));

        self
    }

    /// Apply a new coord transformation object and returns a new drawing area
    pub fn apply_coord_spec<CT>(&self, coord_spec: CT) -> DrawingArea<CT> {
        DrawingArea {
            rect: self.rect.clone(),
            coord: coord_spec,
        }
    }

    /// Create a margin for the given drawing area and returns the new drawing area
    pub fn margin<ST: SizeDesc, SB: SizeDesc, SL: SizeDesc, SR: SizeDesc>(
        &self,
        top: ST,
        bottom: SB,
        left: SL,
        right: SR,
    ) -> DrawingArea<Shift> {
        let left = left.in_pixels(self);
        let right = right.in_pixels(self);
        let top = top.in_pixels(self);
        let bottom = bottom.in_pixels(self);
        DrawingArea {
            rect: Rect {
                x0: self.rect.x0 + left,
                y0: self.rect.y0 + top,
                x1: self.rect.x1 - right,
                y1: self.rect.y1 - bottom,
            },
            coord: Shift((self.rect.x0 + left, self.rect.y0 + top)),
        }
    }

    /// Split the drawing area vertically
    pub fn split_vertically<S: SizeDesc>(&self, y: S) -> (Self, Self) {
        let y = y.in_pixels(self);
        let split_point = [y + self.rect.y0];
        let mut ret = self.rect.split(split_point.iter(), true).map(|rect| Self {
            rect: rect.clone(),
            coord: Shift((rect.x0, rect.y0)),
        });

        (ret.next().unwrap(), ret.next().unwrap())
    }

    /// Split the drawing area horizontally
    pub fn split_horizontally<S: SizeDesc>(&self, x: S) -> (Self, Self) {
        let x = x.in_pixels(self);
        let split_point = [x + self.rect.x0];
        let mut ret = self.rect.split(split_point.iter(), false).map(|rect| Self {
            rect: rect.clone(),
            coord: Shift((rect.x0, rect.y0)),
        });

        (ret.next().unwrap(), ret.next().unwrap())
    }

    /// Split the drawing area evenly
    pub fn split_evenly(&self, (row, col): (usize, usize)) -> Vec<Self> {
        self.rect
            .split_evenly((row, col))
            .map(|rect| Self {
                rect: rect.clone(),
                coord: Shift((rect.x0, rect.y0)),
            })
            .collect()
    }

    /// Split the drawing area into a grid with specified breakpoints on both X axis and Y axis
    pub fn split_by_breakpoints<
        XSize: SizeDesc,
        YSize: SizeDesc,
        XS: AsRef<[XSize]>,
        YS: AsRef<[YSize]>,
    >(
        &self,
        xs: XS,
        ys: YS,
    ) -> Vec<Self> {
        self.rect
            .split_grid(
                xs.as_ref().iter().map(|x| x.in_pixels(self)),
                ys.as_ref().iter().map(|x| x.in_pixels(self)),
            )
            .map(|rect| Self {
                rect: rect.clone(),
                coord: Shift((rect.x0, rect.y0)),
            })
            .collect()
    }

    /// Draw a title of the drawing area and return the remaining drawing area
    pub fn titled<'b, DB: DrawingBackend, S: Into<TextStyle<'b>>>(
        &self,
        backend: &mut DB,
        text: &str,
        style: S,
    ) -> Result<Self, DrawingAreaError> {
        let style = style.into();

        let x_padding = (self.rect.x1 - self.rect.x0) / 2;

        let (_, text_h) = self.estimate_text_size(backend, text, &style)?;
        let y_padding = (text_h / 2).min(5) as i32;

        let style = style.pos(Pos::new(HPos::Center, VPos::Top));

        self.backend_ops(backend, |b| {
            b.draw_text(
                text,
                style.clone().into(),
                (self.rect.x0 + x_padding, self.rect.y0 + y_padding),
            )
        })?;

        Ok(Self {
            rect: Rect {
                x0: self.rect.x0,
                y0: self.rect.y0 + y_padding * 2 + text_h as i32,
                x1: self.rect.x1,
                y1: self.rect.y1,
            },
            coord: Shift((self.rect.x0, self.rect.y0 + y_padding * 2 + text_h as i32)),
        })
    }

    /// Draw text on the drawing area
    pub fn draw_text<DB: DrawingBackend>(
        &self,
        backend: &mut DB,
        text: &str,
        style: &TextStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingAreaError> {
        self.backend_ops(backend, |b| {
            b.draw_text(
                text,
                style.clone().into(),
                (pos.0 + self.rect.x0, pos.1 + self.rect.y0),
            )
        })
    }
}

impl<CT> DrawingArea<CT> {
    /// Returns the coordinates by value
    pub fn into_coord_spec(self) -> CT {
        self.coord
    }

    /// Returns the coordinates by reference
    pub fn as_coord_spec(&self) -> &CT {
        &self.coord
    }

    /// Returns the coordinates by mutable reference
    pub fn as_coord_spec_mut(&mut self) -> &mut CT {
        &mut self.coord
    }
}

#[cfg(test)]
mod drawing_area_tests {
    use crate::prelude::*;
    #[test]
    fn test_filling() {
        let mut backend = MockedBackend::new(1024, 768);
        {
            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, WHITE.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (0, 0));
                assert_eq!(d, (1024, 768));
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        }
        let drawing_area = backend.to_drawing_area();

        drawing_area
            .fill(&mut backend, &WHITE)
            .expect("Drawing Failure");
    }

    #[test]
    fn test_split_evenly() {
        let colors = vec![
            &RED, &BLUE, &YELLOW, &WHITE, &BLACK, &MAGENTA, &CYAN, &BLUE, &RED,
        ];
        let mut backend = MockedBackend::new(902, 900);
        {
            for col in 0..3 {
                for row in 0..3 {
                    let colors = colors.clone();
                    backend.check_draw_rect(move |c, _, f, u, d| {
                        assert_eq!(c, colors[col * 3 + row].to_rgba());
                        assert_eq!(f, true);
                        assert_eq!(u, (300 * row as i32 + 2.min(row) as i32, 300 * col as i32));
                        assert_eq!(
                            d,
                            (
                                300 + 300 * row as i32 + 2.min(row + 1) as i32,
                                300 + 300 * col as i32
                            )
                        );
                    });
                }
            }
            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 9);
                assert_eq!(b.draw_count, 9);
            });
        }
        let drawing_area = backend.to_drawing_area();

        drawing_area
            .split_evenly((3, 3))
            .iter_mut()
            .zip(colors.iter())
            .for_each(|(d, c)| {
                d.fill(&mut backend, *c).expect("Drawing Failure");
            });
    }

    #[test]
    fn test_split_horizontally() {
        let mut backend = MockedBackend::new(1024, 768);
        {
            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, RED.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (0, 0));
                assert_eq!(d, (345, 768));
            });

            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, BLUE.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (345, 0));
                assert_eq!(d, (1024, 768));
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 2);
                assert_eq!(b.draw_count, 2);
            });
        }
        let drawing_area = backend.to_drawing_area();

        let (left, right) = drawing_area.split_horizontally(345);
        left.fill(&mut backend, &RED).expect("Drawing Error");
        right.fill(&mut backend, &BLUE).expect("Drawing Error");
    }

    #[test]
    fn test_split_vertically() {
        let mut backend = MockedBackend::new(1024, 768);
        {
            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, RED.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (0, 0));
                assert_eq!(d, (1024, 345));
            });

            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, BLUE.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (0, 345));
                assert_eq!(d, (1024, 768));
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 2);
                assert_eq!(b.draw_count, 2);
            });
        }
        let drawing_area = backend.to_drawing_area();

        let (left, right) = drawing_area.split_vertically(345);
        left.fill(&mut backend, &RED).expect("Drawing Error");
        right.fill(&mut backend, &BLUE).expect("Drawing Error");
    }

    #[test]
    fn test_split_grid() {
        let colors = vec![
            &RED, &BLUE, &YELLOW, &WHITE, &BLACK, &MAGENTA, &CYAN, &BLUE, &RED,
        ];
        let breaks: [i32; 5] = [100, 200, 300, 400, 500];

        for nxb in 0..=5 {
            for nyb in 0..=5 {
                let mut backend = MockedBackend::new(1024, 768);
                {
                    for row in 0..=nyb {
                        for col in 0..=nxb {
                            let get_bp = |full, limit, id| {
                                (if id == 0 {
                                    0
                                } else if id > limit {
                                    full
                                } else {
                                    breaks[id as usize - 1]
                                }) as i32
                            };

                            let expected_u = (get_bp(1024, nxb, col), get_bp(768, nyb, row));
                            let expected_d =
                                (get_bp(1024, nxb, col + 1), get_bp(768, nyb, row + 1));
                            let expected_color =
                                colors[(row * (nxb + 1) + col) as usize % colors.len()];

                            backend.check_draw_rect(move |c, _, f, u, d| {
                                assert_eq!(c, expected_color.to_rgba());
                                assert_eq!(f, true);
                                assert_eq!(u, expected_u);
                                assert_eq!(d, expected_d);
                            });
                        }
                    }

                    backend.drop_check(move |b| {
                        assert_eq!(b.num_draw_rect_call, ((nxb + 1) * (nyb + 1)) as u32);
                        assert_eq!(b.draw_count, ((nyb + 1) * (nxb + 1)) as u32);
                    });
                }
                let drawing_area = backend.to_drawing_area();

                let result = drawing_area
                    .split_by_breakpoints(&breaks[0..nxb as usize], &breaks[0..nyb as usize]);
                for i in 0..result.len() {
                    result[i]
                        .fill(&mut backend, colors[i % colors.len()])
                        .expect("Drawing Error");
                }
            }
        }
    }
    #[test]
    fn test_titled() {
        let mut backend = MockedBackend::new(1024, 768);
        {
            backend.check_draw_text(|c, font, size, _pos, text| {
                assert_eq!(c, BLACK.to_rgba());
                assert_eq!(font, "serif");
                assert_eq!(size, 30.0);
                assert_eq!("This is the title", text);
            });
            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, WHITE.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u.0, 0);
                assert!(u.1 > 0);
                assert_eq!(d, (1024, 768));
            });
            backend.drop_check(|b| {
                assert_eq!(b.num_draw_text_call, 1);
                assert_eq!(b.num_draw_rect_call, 1);
                assert_eq!(b.draw_count, 2);
            });
        }
        let drawing_area = backend.to_drawing_area();

        drawing_area
            .titled(&mut backend, "This is the title", ("serif", 30))
            .unwrap()
            .fill(&mut backend, &WHITE)
            .unwrap();
    }

    #[test]
    fn test_margin() {
        let mut backend = MockedBackend::new(1024, 768);
        {
            backend.check_draw_rect(|c, _, f, u, d| {
                assert_eq!(c, WHITE.to_rgba());
                assert_eq!(f, true);
                assert_eq!(u, (3, 1));
                assert_eq!(d, (1024 - 4, 768 - 2));
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        }
        let drawing_area = backend.to_drawing_area();

        drawing_area
            .margin(1, 2, 3, 4)
            .fill(&mut backend, &WHITE)
            .expect("Drawing Failure");
    }

    #[test]
    fn test_ranges() {
        let drawing_area = MockedBackend::new(1024, 768)
            .to_drawing_area()
            .apply_coord_spec(Cartesian2d::<
            crate::coord::types::RangedCoordi32,
            crate::coord::types::RangedCoordu32,
        >::new(-100..100, 0..200, (0..1024, 0..768)));

        let x_range = drawing_area.get_x_range();
        assert_eq!(x_range, -100..100);

        let y_range = drawing_area.get_y_range();
        assert_eq!(y_range, 0..200);
    }

    #[test]
    fn test_relative_size() {
        let drawing_area = MockedBackend::new(1024, 768).to_drawing_area();

        assert_eq!(102.4, drawing_area.relative_to_width(0.1));
        assert_eq!(384.0, drawing_area.relative_to_height(0.5));

        assert_eq!(1024.0, drawing_area.relative_to_width(1.3));
        assert_eq!(768.0, drawing_area.relative_to_height(1.5));

        assert_eq!(0.0, drawing_area.relative_to_width(-0.2));
        assert_eq!(0.0, drawing_area.relative_to_height(-0.5));
    }

    #[test]
    fn test_relative_split() {
        let mut backend = MockedBackend::new(1000, 1200);
        {
            let mut counter = 0;
            backend.check_draw_rect(move |c, _, f, u, d| {
                assert_eq!(f, true);

                match counter {
                    0 => {
                        assert_eq!(c, RED.to_rgba());
                        assert_eq!(u, (0, 0));
                        assert_eq!(d, (300, 600));
                    }
                    1 => {
                        assert_eq!(c, BLUE.to_rgba());
                        assert_eq!(u, (300, 0));
                        assert_eq!(d, (1000, 600));
                    }
                    2 => {
                        assert_eq!(c, GREEN.to_rgba());
                        assert_eq!(u, (0, 600));
                        assert_eq!(d, (300, 1200));
                    }
                    3 => {
                        assert_eq!(c, WHITE.to_rgba());
                        assert_eq!(u, (300, 600));
                        assert_eq!(d, (1000, 1200));
                    }
                    _ => panic!("Too many draw rect"),
                }

                counter += 1;
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 4);
                assert_eq!(b.draw_count, 4);
            });
        }
        let drawing_area = backend.to_drawing_area();

        let split =
            drawing_area.split_by_breakpoints([(30).percent_width()], [(50).percent_height()]);

        split[0].fill(&mut backend, &RED).unwrap();
        split[1].fill(&mut backend, &BLUE).unwrap();
        split[2].fill(&mut backend, &GREEN).unwrap();
        split[3].fill(&mut backend, &WHITE).unwrap();
    }

    #[test]
    fn test_relative_shrink() {
        let mut backend = MockedBackend::new(1000, 1200);
        {
            backend.check_draw_rect(move |_, _, _, u, d| {
                assert_eq!((100, 100), u);
                assert_eq!((300, 700), d);
            });

            backend.drop_check(|b| {
                assert_eq!(b.num_draw_rect_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        }
        let drawing_area = backend
            .to_drawing_area()
            .shrink(((10).percent_width(), 100), (200, (50).percent_height()));

        drawing_area.fill(&mut backend, &RED).unwrap();
    }
}
