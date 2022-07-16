//! The implementation of an actual font implementation
//!
//! This exists since for the image rendering task, we want to use
//! the system font. But in wasm application, we want the browser
//! to handle all the font issue.
//!
//! Thus we need different mechanism for the font implementation

use num_traits::{FromPrimitive, Signed, ToPrimitive};

#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ttf"
))]
mod ttf;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ttf"
))]
use ttf::FontDataInternal;

#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    not(feature = "ttf")
))]
mod naive;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    not(feature = "ttf")
))]
use naive::FontDataInternal;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod web;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
use web::FontDataInternal;

mod font_desc;
pub use font_desc::*;

/// Represents a box where a text label can be fit
pub type LayoutBox<C = i32> = ((C, C), (C, C));

pub trait FontData: Clone {
    type ErrorType: Sized + std::error::Error + Clone;
    fn new(family: FontFamily, style: FontStyle) -> Result<Self, Self::ErrorType>;
    fn estimate_layout<C: FromPrimitive>(
        &self,
        size: f64,
        text: &str,
    ) -> Result<LayoutBox<C>, Self::ErrorType>;
    fn draw<
        C: Copy + FromPrimitive + ToPrimitive + Signed,
        E,
        DrawFunc: FnMut(C, C, f32) -> Result<(), E>,
    >(
        &self,
        _pos: (C, C),
        _size: f64,
        _text: &str,
        _draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        panic!("The font implementation is unable to draw text");
    }
}
