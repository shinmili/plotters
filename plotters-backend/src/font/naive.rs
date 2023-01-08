use std::error::Error;

use super::{FontBackend, FontData, FontDesc, LayoutBox};

#[derive(Debug, Clone)]
pub struct FontError;

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "General Error")?;
        Ok(())
    }
}

impl std::error::Error for FontError {}

pub struct NaiveFontBackend;

impl FontBackend for NaiveFontBackend {
    type Font = NaiveFontData;

    fn load_font(&self, desc: &FontDesc) -> Result<Self::Font, Box<dyn Error + Send + Sync>> {
        Ok(NaiveFontData(
            desc.get_family().as_str().into(),
            desc.get_style().as_str().into(),
        ))
    }
}

#[derive(Clone)]
pub struct NaiveFontData(String, String);

impl FontData for NaiveFontData {
    type ErrorType = FontError;

    /// Note: This is only a crude estimatation, since for some backend such as SVG, we have no way to
    /// know the real size of the text anyway. Thus using font-kit is an overkill and doesn't helps
    /// the layout.
    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let em = size / 1.24 / 1.24;
        Ok((
            (0, -em.round() as i32),
            (
                (em * 0.7 * text.len() as f64).round() as i32,
                (em * 0.24).round() as i32,
            ),
        ))
    }
}
