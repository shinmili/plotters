use std::convert::From;

pub use crate::{FontFamily, FontStyle, FontTransform};

/// Describes a font
#[derive(Clone)]
pub struct FontDesc<'a> {
    size: f64,
    family: FontFamily<'a>,
    transform: FontTransform,
    style: FontStyle,
}

impl<'a> FontDesc<'a> {
    /// Create a new font
    ///
    /// - `family`: The font family name
    /// - `size`: The size of the font
    /// - `style`: The font variations
    /// - **returns** The newly created font description
    pub fn new(family: FontFamily<'a>, size: f64, style: FontStyle) -> Self {
        Self {
            size,
            family,
            transform: FontTransform::None,
            style,
        }
    }

    /// Create a new font desc with the same font but different size
    ///
    /// - `size`: The new size to set
    /// - **returns** The newly created font descriptor with a new size
    pub fn resize(&self, size: f64) -> Self {
        Self {
            size,
            family: self.family,
            transform: self.transform.clone(),
            style: self.style,
        }
    }

    /// Set the style of the font
    ///
    /// - `style`: The new style
    /// - **returns** The new font description with this style applied
    pub fn style(&self, style: FontStyle) -> Self {
        Self {
            size: self.size,
            family: self.family,
            transform: self.transform.clone(),
            style,
        }
    }

    /// Set the font transformation
    ///
    /// - `trans`: The new transformation
    /// - **returns** The new font description with this font transformation applied
    pub fn transform(&self, trans: FontTransform) -> Self {
        Self {
            size: self.size,
            family: self.family,
            transform: trans,
            style: self.style,
        }
    }

    /// Get the font transformation description
    pub fn get_transform(&self) -> FontTransform {
        self.transform.clone()
    }

    /// Returns the font family
    pub fn get_family(&self) -> FontFamily {
        self.family
    }

    /// Get the name of the font
    pub fn get_name(&self) -> &str {
        self.family.as_str()
    }

    /// Get the name of the style
    pub fn get_style(&self) -> FontStyle {
        self.style
    }

    /// Get the size of font
    pub fn get_size(&self) -> f64 {
        self.size
    }
}

impl<'a> From<&'a str> for FontDesc<'a> {
    fn from(from: &'a str) -> FontDesc<'a> {
        FontDesc::new(from.into(), 12.0, FontStyle::Normal)
    }
}

impl<'a> From<FontFamily<'a>> for FontDesc<'a> {
    fn from(family: FontFamily<'a>) -> FontDesc<'a> {
        FontDesc::new(family, 12.0, FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>> From<(FontFamily<'a>, T)> for FontDesc<'a> {
    fn from((family, size): (FontFamily<'a>, T)) -> FontDesc<'a> {
        FontDesc::new(family, size.into(), FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>> From<(&'a str, T)> for FontDesc<'a> {
    fn from((typeface, size): (&'a str, T)) -> FontDesc<'a> {
        FontDesc::new(typeface.into(), size.into(), FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>, S: Into<FontStyle>> From<(FontFamily<'a>, T, S)> for FontDesc<'a> {
    fn from((family, size, style): (FontFamily<'a>, T, S)) -> FontDesc<'a> {
        FontDesc::new(family, size.into(), style.into())
    }
}

impl<'a, T: Into<f64>, S: Into<FontStyle>> From<(&'a str, T, S)> for FontDesc<'a> {
    fn from((typeface, size, style): (&'a str, T, S)) -> FontDesc<'a> {
        FontDesc::new(typeface.into(), size.into(), style.into())
    }
}

/// The trait that allows some type turns into a font description
pub trait IntoFont<'a> {
    /// Make the font description from the source type
    fn into_font(self) -> FontDesc<'a>;
}

impl<'a, T: Into<FontDesc<'a>>> IntoFont<'a> for T {
    fn into_font(self) -> FontDesc<'a> {
        self.into()
    }
}
