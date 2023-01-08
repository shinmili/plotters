use super::BackendColor;

/// Describes font family.
/// This can be either a specific font family name, such as "arial",
/// or a general font family class, such as "serif" and "sans-serif"
#[derive(Clone, Copy)]
pub enum FontFamily<'a> {
    /// The system default serif font family
    Serif,
    /// The system default sans-serif font family
    SansSerif,
    /// The system default monospace font
    Monospace,
    /// A specific font family name
    Name(&'a str),
}

impl<'a> FontFamily<'a> {
    /// Make a CSS compatible string for the font family name.
    /// This can be used as the value of `font-family` attribute in SVG.
    pub fn as_str(&self) -> &str {
        match self {
            FontFamily::Serif => "serif",
            FontFamily::SansSerif => "sans-serif",
            FontFamily::Monospace => "monospace",
            FontFamily::Name(face) => face,
        }
    }
}

impl<'a> From<&'a str> for FontFamily<'a> {
    fn from(from: &'a str) -> FontFamily<'a> {
        match from.to_lowercase().as_str() {
            "serif" => FontFamily::Serif,
            "sans-serif" => FontFamily::SansSerif,
            "monospace" => FontFamily::Monospace,
            _ => FontFamily::Name(from),
        }
    }
}

/// Text anchor attributes are used to properly position the text.
///
/// # Examples
///
/// In the example below, the text anchor (X) position is `Pos::new(HPos::Right, VPos::Center)`.
/// ```text
///    ***** X
/// ```
/// The position is always relative to the text regardless of its rotation.
/// In the example below, the text has style
/// `style.transform(FontTransform::Rotate90).pos(Pos::new(HPos::Center, VPos::Top))`.
/// ```text
///        *
///        *
///        * X
///        *
///        *
/// ```
pub mod text_anchor {
    /// The horizontal position of the anchor point relative to the text.
    #[derive(Clone, Copy)]
    pub enum HPos {
        /// Anchor point is on the left side of the text
        Left,
        /// Anchor point is on the right side of the text
        Right,
        /// Anchor point is in the horizontal center of the text
        Center,
    }

    /// The vertical position of the anchor point relative to the text.
    #[derive(Clone, Copy)]
    pub enum VPos {
        /// Anchor point is on the top of the text
        Top,
        /// Anchor point is in the vertical center of the text
        Center,
        /// Anchor point is on the bottom of the text
        Bottom,
    }

    /// The text anchor position.
    #[derive(Clone, Copy)]
    pub struct Pos {
        /// The horizontal position of the anchor point
        pub h_pos: HPos,
        /// The vertical position of the anchor point
        pub v_pos: VPos,
    }

    impl Pos {
        /// Create a new text anchor position.
        ///
        /// - `h_pos`: The horizontal position of the anchor point
        /// - `v_pos`: The vertical position of the anchor point
        /// - **returns** The newly created text anchor position
        ///
        /// ```rust
        /// use plotters_backend::text_anchor::{Pos, HPos, VPos};
        ///
        /// let pos = Pos::new(HPos::Left, VPos::Top);
        /// ```
        pub fn new(h_pos: HPos, v_pos: VPos) -> Self {
            Pos { h_pos, v_pos }
        }

        /// Create a default text anchor position (top left).
        ///
        /// - **returns** The default text anchor position
        ///
        /// ```rust
        /// use plotters_backend::text_anchor::{Pos, HPos, VPos};
        ///
        /// let pos = Pos::default();
        /// ```
        pub fn default() -> Self {
            Pos {
                h_pos: HPos::Left,
                v_pos: VPos::Top,
            }
        }
    }
}

/// Specifying text transformations
#[derive(Clone, Copy)]
pub enum FontTransform {
    /// Nothing to transform
    None,
    /// Rotating the text 90 degree clockwise
    Rotate90,
    /// Rotating the text 180 degree clockwise
    Rotate180,
    /// Rotating the text 270 degree clockwise
    Rotate270,
    /// Rotating the text to an arbitrary degree clockwise
    RotateAngle(f32),
}

impl FontTransform {
    /// Transform the coordinate to perform the rotation
    ///
    /// - `x`: The x coordinate in pixels before transform
    /// - `y`: The y coordinate in pixels before transform
    /// - **returns**: The coordinate after transform
    pub fn transform(&self, x: i32, y: i32) -> (i32, i32) {
        match self {
            FontTransform::None => (x, y),
            FontTransform::Rotate90 => (-y, x),
            FontTransform::Rotate180 => (-x, -y),
            FontTransform::Rotate270 => (y, -x),
            FontTransform::RotateAngle(angle) => {
                let (x, y) = (x as f32, y as f32);
                let (sin, cos) = angle.to_radians().sin_cos();
                let (x, y) = (x * cos - y * sin, x * sin + y * cos);
                (x.round() as i32, y.round() as i32)
            }
        }
    }
}

/// Describes the font style. Such as Italic, Oblique, etc.
#[derive(Clone, Copy)]
pub enum FontStyle {
    /// The normal style
    Normal,
    /// The oblique style
    Oblique,
    /// The italic style
    Italic,
    /// The bold style
    Bold,
}

impl FontStyle {
    /// Convert the font style into a CSS compatible string which can be used in `font-style` attribute.
    pub fn as_str(&self) -> &str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique => "oblique",
            FontStyle::Bold => "bold",
        }
    }
}

impl<'a> From<&'a str> for FontStyle {
    fn from(from: &'a str) -> FontStyle {
        match from.to_lowercase().as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            "bold" => FontStyle::Bold,
            _ => FontStyle::Normal,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BackendTextStyle<'a> {
    pub color: BackendColor,
    pub size: f64,
    pub transform: FontTransform,
    pub style: FontStyle,
    pub anchor: text_anchor::Pos,
    pub family: FontFamily<'a>,
}

#[cfg(test)]
mod tests {
    use crate::FontTransform;

    #[test]
    fn text_rotation_works() {
        assert_eq!(FontTransform::None.transform(1, 0), (1, 0));
        assert_eq!(FontTransform::Rotate90.transform(1, 0), (0, 1));
        assert_eq!(FontTransform::Rotate180.transform(1, 0), (-1, 0));
        assert_eq!(FontTransform::Rotate270.transform(1, 0), (0, -1));

        assert_eq!(FontTransform::RotateAngle(45f32).transform(10, 0), (7, 7));
        assert_eq!(FontTransform::RotateAngle(45f32).transform(0, 10), (-7, 7));
    }
}
