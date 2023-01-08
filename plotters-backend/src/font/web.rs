use std::error::Error;

use super::{FontData, LayoutBox};
use crate::{FontBackend, FontDesc};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};

#[derive(Debug, Clone)]
pub enum FontError {
    UnknownError,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            _ => write!(fmt, "Unknown error"),
        }
    }
}

impl Error for FontError {}

pub struct WebFontBackend;

impl FontBackend for WebFontBackend {
    type Font = WebFontData;

    fn load_font(&self, desc: &FontDesc) -> Result<Self::Font, Box<dyn Error + Send + Sync>> {
        Ok(WebFontData(
            desc.get_family().as_str().into(),
            desc.get_style().as_str().into(),
        ))
    }
}

#[derive(Clone)]
pub struct WebFontData(String, String);

impl FontData for WebFontData {
    type ErrorType = FontError;

    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let span = document.create_element("span").unwrap();
        span.set_text_content(Some(text));
        span.set_attribute("style", &format!("display: inline-block; font-family:{}; font-size: {}px; position: fixed; top: 100%", self.0, size)).unwrap();
        let span = span.into();
        body.append_with_node_1(&span).unwrap();
        let elem = JsCast::dyn_into::<HtmlElement>(span).unwrap();
        let height = elem.offset_height() as i32;
        let width = elem.offset_width() as i32;
        elem.remove();
        Ok(((0, 0), (width, height)))
    }
}
