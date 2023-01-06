/// The color type that is used by all the backend
#[derive(Clone, Copy)]
pub struct BackendColor {
    pub alpha: f64,
    pub rgb: (u8, u8, u8),
}

impl BackendColor {
    #[inline(always)]
    pub fn mix(&self, alpha: f64) -> Self {
        Self {
            alpha: self.alpha * alpha,
            rgb: self.rgb,
        }
    }
}

/// The style data for the backend drawing API
#[derive(Clone, Copy)]
pub struct BackendStyle {
    /// The color of current style
    pub color: BackendColor,
    /// The stroke width of current style
    pub stroke_width: u32,
}

impl From<BackendColor> for BackendStyle {
    fn from(value: BackendColor) -> Self {
        BackendStyle {
            color: value,
            stroke_width: 1,
        }
    }
}
