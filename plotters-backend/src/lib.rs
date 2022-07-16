/*!
  The Plotters backend API crate. This is a part of Plotters, the Rust drawing and plotting library, for more details regarding the entire
  Plotters project, please check the [main crate](https://crates.io/crates/plotters).

  This is the crate that used as the connector between Plotters and different backend crates. Since Plotters 0.3, all the backends has been
  hosted as seperate crates for the usability and maintainability reasons.

  At the same time, Plotters is now supporting third-party backends and all the backends are now supports "plug-and-play":
  To use a external backend, just depends on both the Plotters main crate and the third-party backend crate.

  # Notes for implementing Backend for Plotters

  To create a new Plotters backend, this crate should be imported to the crate and the trait [DrawingBackend](trait.DrawingBackend.html) should
  be implemented. It's highly recommended that the third-party backend uses `plotters-backend` by version specification `^x.y.*`.
  For more details, see the [compatibility note](#compatibility-note).

  If the backend only implements [DrawingBackend::draw_pixel](trait.DrawingBackend.html#tymethod.draw_pixel), the default CPU rasterizer will be
  used to give your backend ability of drawing different shapes. For those backend that supports advanced drawing instructions, such as, GPU
  acelerated shape drawing, all the provided trait method can be overriden from the specific backend code.

  If your backend have text rendering ability, you may want to override the [DrawingBackend::estimate_text_size](trait.DrawingBackend.html#tymethod.estimate_text_size)
  to avoid wrong spacing, since the Plotters default text handling code may behaves differently from the backend in terms of text rendering.

  ## Animated or Realtime Rendering
  Backend might render the image realtimely/animated, for example, a GTK backend for realtime display or a GIF image rendering. To support these
  features, you need to play with `ensure_prepared` and `present` method. The following figure illustrates how Plotters operates a drawing backend.

  - `ensure_prepared` - Called before each time when plotters want to draw. This function should initialize the backend for current frame, if the backend is already prepared
     for a frame, this function should simply do nothing.
  - `present` - Called when plotters want to finish current frame drawing


  ```text
                                        .ensure_prepared() &&
    +-------------+    +-------------+    .draw_pixels()             +--------------+   drop
    |Start drwaing|--->|Ready to draw| ------------------------+---->|Finish 1 frame| --------->
    +-------------+    +-------------+                         |     +--------------+
           ^                  ^                                |            |
           |                  +------------------------------- +            |
           |                            continue drawing                    |
           +----------------------------------------------------------------+
                                start render the next frame
                                        .present()
  ```
  - For both animated and static drawing, `DrawingBackend::present` indicates current frame should be flushed.
  - For both animated and static drawing, `DrawingBackend::ensure_prepared` is called every time when plotters need to draw.
  - For static drawing, the `DrawingBackend::present` is only called once manually, or from the Drop impl for the backend.
  - For dynamic drawing, frames are defined by invocation of `DrawingBackend::present`, everything prior the invocation should belongs to previous frame

  # Compatibility Note
  Since Plotters v0.3, plotters use the "plug-and-play" schema to import backends, this requires both Plotters and the backend crates depdens on a
  same version of `plotters-backend` crate. This crate (`plotters-backend`) will enforce that any revision (means the last number in a version number)
  won't contains breaking change - both on the Plotters side and backend side.

  Plotters main crate is always importing the backend crate with version specification `plotters-backend = "^<major>.<minor>*"`.
  It's highly recommended that all the external crates follows the same rule to import `plotters-backend` depdendency, to avoid protential breaking
  caused by `plotters-backend` crates gets a revision update.

  We also impose a versioning rule with `plotters` and some backends:
  The compatible main crate (`plotters`) and this crate (`plotters-backend`) are always use the same major and minor version number.
  All the plotters main crate and second-party backends with version "x.y.*" should be compatible, and they should depens on the latest version of `plotters-backend x.y.*`

*/
use std::error::Error;

pub mod rasterizer;
mod style;
mod text;

pub use style::{BackendColor, BackendStyle};
pub use text::{text_anchor, BackendTextStyle, FontFamily, FontStyle, FontTransform};

/// A coordinate in the pixel-based backend. The coordinate follows the framebuffer's convention,
/// which defines the top-left point as (0, 0).
pub type BackendCoord = (i32, i32);

/// The error produced by a drawing backend.
#[derive(Debug)]
pub enum DrawingErrorKind<E: Error + Send + Sync> {
    /// A drawing backend error
    DrawingError(E),
    /// A font rendering error
    FontError(Box<dyn Error + Send + Sync + 'static>),
}

impl<E: Error + Send + Sync> std::fmt::Display for DrawingErrorKind<E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DrawingErrorKind::DrawingError(e) => write!(fmt, "Drawing backend error: {}", e),
            DrawingErrorKind::FontError(e) => write!(fmt, "Font loading error: {}", e),
        }
    }
}

impl<E: Error + Send + Sync> Error for DrawingErrorKind<E> {}

/// The drawing backend trait, which implements the low-level drawing APIs.
///
/// - `C`: The type of a coordinate value.
pub trait DrawingBackend<C = i32>: Sized {
    /// The error type reported by the backend
    type ErrorType: Error + Send + Sync;

    /// Get the dimension of the drawing backend in pixels
    fn get_size(&self) -> (u32, u32);

    /// Ensure the backend is ready to draw
    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Finalize the drawing step and present all the changes.
    /// This is used as the real-time rendering support.
    /// The backend may implement in the following way, when `ensure_prepared` is called
    /// it checks if it needs a fresh buffer and `present` is called rendering all the
    /// pending changes on the screen.
    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a pixel on the drawing backend
    /// - `point`: The backend pixel-based coordinate to draw
    /// - `color`: The color of the pixel
    fn draw_pixel(
        &mut self,
        point: (C, C),
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a line on the drawing backend
    /// - `from`: The start point of the line
    /// - `to`: The end point of the line
    /// - `style`: The style of the line
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (C, C),
        to: (C, C),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a rectangle on the drawing backend
    /// - `upper_left`: The coordinate of the upper-left corner of the rect
    /// - `bottom_right`: The coordinate of the bottom-right corner of the rect
    /// - `style`: The style
    /// - `fill`: If the rectangle should be filled
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: (C, C),
        bottom_right: (C, C),
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a path on the drawing backend
    /// - `path`: The iterator of key points of the path
    /// - `style`: The style of the path
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = (C, C)>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a circle on the drawing backend
    /// - `center`: The center coordinate of the circle
    /// - `radius`: The radius of the circle
    /// - `style`: The style of the shape
    /// - `fill`: If the circle should be filled
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: (C, C),
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = (C, C)>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a text on the drawing backend
    /// - `text`: The text to draw
    /// - `style`: The text style
    /// - `pos` : The text anchor point
    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: (C, C),
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Estimate the size of the horizontal text if rendered on this backend.
    /// This is important because some of the backend may not have font ability.
    /// Thus this allows those backend reports proper value rather than ask the
    /// font rasterizer for that.
    ///
    /// - `text`: The text to estimate
    /// - `font`: The font to estimate
    /// - *Returns* The estimated text size
    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let layout = style
            .layout_box(text)
            .map_err(|e| DrawingErrorKind::FontError(Box::new(e)))?;
        Ok((
            ((layout.1).0 - (layout.0).0) as u32,
            ((layout.1).1 - (layout.0).1) as u32,
        ))
    }

    /// Blit a bitmap on to the backend.
    ///
    /// - `text`: pos the left upper conner of the bitmap to blit
    /// - `src`: The source of the image
    ///
    /// TODO: The default implementation of bitmap blitting assumes that the bitmap is RGB, but
    /// this may not be the case. But for bitmap backend it's actually ok if we use the bitmap
    /// element that matches the pixel format, but we need to fix this.
    fn blit_bitmap(
        &mut self,
        pos: (C, C),
        src_size: (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;
}
