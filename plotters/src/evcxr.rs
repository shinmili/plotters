use crate::coord::Shift;
use crate::drawing::{DrawingArea, ToDrawingArea};
use plotters_svg::SVGBackend;

/// The wrapper for the generated SVG
pub struct SVGWrapper(String, String);

impl SVGWrapper {
    /// Displays the contents of the `SVGWrapper` struct.
    pub fn evcxr_display(&self) {
        println!("{:?}", self);
    }
    /// Sets the style of the `SVGWrapper` struct.
    pub fn style<S: Into<String>>(mut self, style: S) -> Self {
        self.1 = style.into();
        self
    }
}

impl std::fmt::Debug for SVGWrapper {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let svg = self.0.as_str();
        write!(
            formatter,
            "EVCXR_BEGIN_CONTENT text/html\n<div style=\"{}\">{}</div>\nEVCXR_END_CONTENT",
            self.1, svg
        )
    }
}

/// Start drawing an evcxr figure
pub fn evcxr_figure<
    Draw: FnOnce(&mut SVGBackend, DrawingArea<Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    size: (u32, u32),
    draw: Draw,
) -> SVGWrapper {
    let mut buffer = "".to_string();
    {
        let mut backend = SVGBackend::with_string(&mut buffer, size);
        let root = backend.to_drawing_area();
        draw(&mut backend, root).expect("Drawing failure");
    }
    SVGWrapper(buffer, "".to_string())
}
