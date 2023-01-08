use std::error::Error;

/// The implementation of an actual font implementation
///
/// This exists since for the image rendering task, we want to use
/// the system font. But in wasm application, we want the browser
/// to handle all the font issue.
///
/// Thus we need different mechanism for the font implementation

#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ttf"
))]
mod ttf;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ttf"
))]
pub use ttf::{TtfFontBackend as DefaultFontBackend, TtfFontData};

#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    not(feature = "ttf")
))]
mod naive;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    not(feature = "ttf")
))]
pub use naive::{NaiveFontBackend as DefaultFontBackend, NaiveFontData};

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod web;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub use web::{WebFontBackend as DefaultFontBackend, WebFontData};

mod font_desc;
pub use font_desc::*;

/// Abstraction over different font loaders.
pub trait FontBackend {
    type Font: FontData;

    fn load_font(&self, desc: &FontDesc) -> Result<Self::Font, Box<dyn Error + Send + Sync>>;
}

/// Represents a box where a text label can be fit
pub type LayoutBox = ((i32, i32), (i32, i32));

pub trait FontData: Clone {
    type ErrorType: Sized + std::error::Error + Clone + Send + Sync + 'static;
    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType>;
    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        _pos: (i32, i32),
        _size: f64,
        _text: &str,
        _draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        panic!("The font implementation is unable to draw text");
    }
}
