use super::{FontData, FontFamily, FontStyle, LayoutBox};
use num_traits::FromPrimitive;

#[derive(Debug, Clone)]
pub struct FontError;

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "General Error")?;
        Ok(())
    }
}

impl std::error::Error for FontError {}

#[derive(Clone)]
pub struct FontDataInternal(String, String);

impl FontData for FontDataInternal {
    type ErrorType = FontError;
    fn new(family: FontFamily, style: FontStyle) -> Result<Self, FontError> {
        Ok(FontDataInternal(
            family.as_str().into(),
            style.as_str().into(),
        ))
    }

    /// Note: This is only a crude estimatation, since for some backend such as SVG, we have no way to
    /// know the real size of the text anyway. Thus using font-kit is an overkill and doesn't helps
    /// the layout.
    fn estimate_layout<C: FromPrimitive>(
        &self,
        size: f64,
        text: &str,
    ) -> Result<LayoutBox<C>, Self::ErrorType> {
        let em = size / 1.24 / 1.24;
        Ok((
            (C::from_i32(0).unwrap(), C::from_f64(-em).unwrap()),
            (
                C::from_f64(em * 0.7 * text.len() as f64).unwrap(),
                C::from_f64(em * 0.24).unwrap(),
            ),
        ))
    }
}
