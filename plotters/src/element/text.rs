use std::i32;
use std::{borrow::Borrow, error::Error};

use super::{Drawable, PointCollection};
use crate::style::TextStyle;
use plotters_backend::{
    BackendCoord, DrawingBackend, DrawingErrorKind, FontBackend, FontData, LayoutBox,
};

/// A single line text element. This can be owned or borrowed string, dependents on
/// `String` or `str` moved into.
pub struct Text<'a, Coord, T: Borrow<str>> {
    text: T,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord, T: Borrow<str>> Text<'a, Coord, T> {
    /// Create a new text element
    /// - `text`: The text for the element
    /// - `points`: The upper left conner for the text element
    /// - `style`: The text style
    /// - Return the newly created text element
    pub fn new<S: Into<TextStyle<'a>>>(text: T, points: Coord, style: S) -> Self {
        Self {
            text,
            coord: points,
            style: style.into(),
        }
    }
}

impl<'b, 'a, Coord: 'a, T: Borrow<str> + 'a> PointCollection<'a, Coord> for &'a Text<'b, Coord, T> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.coord)
    }
}

impl<'a, Coord: 'a, T: Borrow<str>> Drawable for Text<'a, Coord, T> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut dyn DrawingBackend,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        if let Some(a) = points.next() {
            return backend.draw_text(self.text.borrow(), self.style.clone().into(), a);
        }
        Ok(())
    }
}

/// An multi-line text element. The `Text` element allows only single line text
/// and the `MultiLineText` supports drawing multiple lines
pub struct MultiLineText<'a, Coord, T: Borrow<str>> {
    lines: Vec<T>,
    coord: Coord,
    style: TextStyle<'a>,
    line_height: f64,
}

impl<'a, Coord, T: Borrow<str>> MultiLineText<'a, Coord, T> {
    /// Create an empty multi-line text element.
    /// Lines can be append to the empty multi-line by calling `push_line` method
    ///
    /// `pos`: The upper left corner
    /// `style`: The style of the text
    pub fn new<S: Into<TextStyle<'a>>>(pos: Coord, style: S) -> Self {
        MultiLineText {
            lines: vec![],
            coord: pos,
            style: style.into(),
            line_height: 1.25,
        }
    }

    /// Set the line height of the multi-line text element
    pub fn set_line_height(&mut self, value: f64) -> &mut Self {
        self.line_height = value;
        self
    }

    /// Push a new line into the given multi-line text
    /// `line`: The line to be pushed
    pub fn push_line<L: Into<T>>(&mut self, line: L) {
        self.lines.push(line.into());
    }

    /// Estimate the multi-line text element's dimension
    pub fn estimate_dimension(
        &self,
        font_backend: &impl FontBackend,
    ) -> Result<(i32, i32), Box<dyn Error + Send + Sync>> {
        let (mut mx, mut my) = (0, 0);

        for ((x, y), t) in self.layout_lines((0, 0)).zip(self.lines.iter()) {
            let (dx, dy) = self.compute_bounding_box_size(t.borrow(), font_backend)?;
            mx = mx.max(x + dx as i32);
            my = my.max(y + dy as i32);
        }

        Ok((mx, my))
    }

    /// Move the location to the specified location
    pub fn relocate(&mut self, coord: Coord) {
        self.coord = coord
    }

    fn layout_lines(&self, (x0, y0): BackendCoord) -> impl Iterator<Item = BackendCoord> {
        let font_height = self.style.font.get_size();
        let actual_line_height = font_height * self.line_height;
        (0..self.lines.len() as u32).map(move |idx| {
            let y = f64::from(y0) + f64::from(idx) * actual_line_height;
            // TODO: Support text alignment as well, currently everything is left aligned
            let x = f64::from(x0);
            (x.round() as i32, y.round() as i32)
        })
    }

    fn compute_bounding_box_size(
        &self,
        text: &str,
        font_backend: &impl FontBackend,
    ) -> Result<(u32, u32), Box<dyn Error + Send + Sync>> {
        let font = font_backend.load_font(&self.style.font)?;
        let ((min_x, min_y), (max_x, max_y)) = font
            .estimate_layout(self.style.font.get_size(), text.borrow())
            .map_err(|e| Box::new(e))?;
        let (w, h) = self
            .style
            .font
            .get_transform()
            .transform(max_x - min_x, max_y - min_y);

        Ok((w.unsigned_abs(), h.unsigned_abs()))
    }
}

impl<'a, T: Borrow<str>> MultiLineText<'a, BackendCoord, T> {
    /// Compute the line layout
    pub fn compute_line_layout(
        &self,
        font_backend: &impl FontBackend,
    ) -> Result<Vec<LayoutBox>, Box<dyn Error + Send + Sync>> {
        let mut ret = vec![];
        for ((x, y), t) in self.layout_lines(self.coord).zip(self.lines.iter()) {
            let (dx, dy) = self.compute_bounding_box_size(t.borrow(), font_backend)?;
            ret.push(((x, y), (x + dx as i32, y + dy as i32)));
        }
        Ok(ret)
    }
}

impl<'a, Coord> MultiLineText<'a, Coord, &'a str> {
    /// Parse a multi-line text into an multi-line element.
    ///
    /// `text`: The text that is parsed
    /// `pos`: The position of the text
    /// `style`: The style for this text
    pub fn from_str<ST: Into<&'a str>, S: Into<TextStyle<'a>>>(
        text: ST,
        pos: Coord,
        style: S,
    ) -> Self {
        let text = text.into();
        let mut ret = MultiLineText::new(pos, style);
        ret.lines = text.lines().collect();
        ret
    }
}

impl<'a, Coord> MultiLineText<'a, Coord, String> {
    /// Parse a multi-line text into an multi-line element.
    ///
    /// `text`: The text that is parsed
    /// `pos`: The position of the text
    /// `style`: The style for this text
    pub fn from_string<S: Into<TextStyle<'a>>>(text: String, pos: Coord, style: S) -> Self {
        let mut ret = MultiLineText::new(pos, style);
        ret.lines = text.lines().map(|l| l.to_string()).collect();
        ret
    }
}

impl<'b, 'a, Coord: 'a, T: Borrow<str> + 'a> PointCollection<'a, Coord>
    for &'a MultiLineText<'b, Coord, T>
{
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.coord)
    }
}

impl<'a, Coord: 'a, T: Borrow<str>> Drawable for MultiLineText<'a, Coord, T> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut dyn DrawingBackend,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind> {
        if let Some(a) = points.next() {
            for (point, text) in self.layout_lines(a).zip(self.lines.iter()) {
                backend.draw_text(text.borrow(), self.style.clone().into(), point)?;
            }
        }
        Ok(())
    }
}
