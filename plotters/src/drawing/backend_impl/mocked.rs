use crate::coord::Shift;
use crate::drawing::area::IntoDrawingArea;
use crate::drawing::DrawingArea;
use crate::style::RGBAColor;
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

use std::collections::VecDeque;

pub fn check_color(left: BackendColor, right: RGBAColor) {
    assert_eq!(
        RGBAColor(left.rgb.0, left.rgb.1, left.rgb.2, left.alpha),
        right
    );
}

pub struct MockedBackend {
    height: u32,
    width: u32,
    init_count: u32,
    pub draw_count: u32,
    pub num_draw_pixel_call: u32,
    pub num_draw_line_call: u32,
    pub num_draw_rect_call: u32,
    pub num_draw_circle_call: u32,
    pub num_draw_text_call: u32,
    pub num_draw_path_call: u32,
    pub num_fill_polygon_call: u32,
    check_draw_pixel: VecDeque<Box<dyn FnMut(RGBAColor, BackendCoord)>>,
    check_draw_line: VecDeque<Box<dyn FnMut(RGBAColor, i32, BackendCoord, BackendCoord)>>,
    check_draw_rect: VecDeque<Box<dyn FnMut(RGBAColor, i32, bool, BackendCoord, BackendCoord)>>,
    check_draw_path: VecDeque<Box<dyn FnMut(RGBAColor, i32, Vec<BackendCoord>)>>,
    check_draw_circle: VecDeque<Box<dyn FnMut(RGBAColor, i32, bool, BackendCoord, i32)>>,
    check_draw_text: VecDeque<Box<dyn FnMut(RGBAColor, &str, f64, BackendCoord, &str)>>,
    check_fill_polygon: VecDeque<Box<dyn FnMut(RGBAColor, Vec<BackendCoord>)>>,
    drop_check: Option<Box<dyn FnMut(&Self)>>,
}

macro_rules! def_set_checker_func {
    (drop_check, $($param:ty),*) => {
        pub fn drop_check<T: FnMut($($param,)*) + 'static>(&mut self, check:T) -> &mut Self {
            self.drop_check = Some(Box::new(check));
            self
        }
    };
    ($name:ident, $($param:ty),*) => {
        pub fn $name<T: FnMut($($param,)*) + 'static>(&mut self, check:T) -> &mut Self {
            self.$name.push_back(Box::new(check));
            self
        }
    }
}

impl MockedBackend {
    pub fn new(width: u32, height: u32) -> Self {
        MockedBackend {
            height,
            width,
            init_count: 0,
            draw_count: 0,
            num_draw_pixel_call: 0,
            num_draw_line_call: 0,
            num_draw_rect_call: 0,
            num_draw_circle_call: 0,
            num_draw_text_call: 0,
            num_draw_path_call: 0,
            num_fill_polygon_call: 0,
            check_draw_pixel: vec![].into(),
            check_draw_line: vec![].into(),
            check_draw_rect: vec![].into(),
            check_draw_path: vec![].into(),
            check_draw_circle: vec![].into(),
            check_draw_text: vec![].into(),
            check_fill_polygon: vec![].into(),
            drop_check: None,
        }
    }

    def_set_checker_func!(check_draw_pixel, RGBAColor, BackendCoord);
    def_set_checker_func!(check_draw_line, RGBAColor, i32, BackendCoord, BackendCoord);
    def_set_checker_func!(
        check_draw_rect,
        RGBAColor,
        i32,
        bool,
        BackendCoord,
        BackendCoord
    );
    def_set_checker_func!(check_draw_path, RGBAColor, i32, Vec<BackendCoord>);
    def_set_checker_func!(check_draw_circle, RGBAColor, i32, bool, BackendCoord, i32);
    def_set_checker_func!(check_draw_text, RGBAColor, &str, f64, BackendCoord, &str);
    def_set_checker_func!(drop_check, &Self);
    def_set_checker_func!(check_fill_polygon, RGBAColor, Vec<BackendCoord>);

    fn check_before_draw(&mut self) {
        self.draw_count += 1;
        //assert_eq!(self.init_count, self.draw_count);
    }
}

#[derive(Debug)]
pub struct MockedError;

impl std::fmt::Display for MockedError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "MockedError")
    }
}

impl std::error::Error for MockedError {}

impl DrawingBackend for MockedBackend {
    type ErrorType = MockedError;

    fn get_size(&self) -> (i32, i32) {
        (self.width as i32, self.height as i32)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<MockedError>> {
        self.init_count += 1;
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<MockedError>> {
        self.init_count = 0;
        self.draw_count = 0;
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_draw_pixel_call += 1;
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_draw_pixel.pop_front() {
            checker(color, point);

            if self.check_draw_pixel.is_empty() {
                self.check_draw_pixel.push_back(checker);
            }
        }
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_draw_line_call += 1;
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_draw_line.pop_front() {
            checker(color, style.stroke_width(), from, to);

            if self.check_draw_line.is_empty() {
                self.check_draw_line.push_back(checker);
            }
        }
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_draw_rect_call += 1;
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_draw_rect.pop_front() {
            checker(color, style.stroke_width(), fill, upper_left, bottom_right);

            if self.check_draw_rect.is_empty() {
                self.check_draw_rect.push_back(checker);
            }
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_draw_path_call += 1;
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_draw_path.pop_front() {
            checker(color, style.stroke_width(), path.into_iter().collect());

            if self.check_draw_path.is_empty() {
                self.check_draw_path.push_back(checker);
            }
        }
        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: i32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_draw_circle_call += 1;
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_draw_circle.pop_front() {
            checker(color, style.stroke_width(), fill, center, radius);

            if self.check_draw_circle.is_empty() {
                self.check_draw_circle.push_back(checker);
            }
        }
        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        self.num_fill_polygon_call += 1;
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        if let Some(mut checker) = self.check_fill_polygon.pop_front() {
            checker(color, path.into_iter().collect());

            if self.check_fill_polygon.is_empty() {
                self.check_fill_polygon.push_back(checker);
            }
        }
        Ok(())
    }

    fn draw_text<S: BackendTextStyle<i32>>(
        &mut self,
        text: &str,
        style: &S,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = style.color();
        let color = RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha);
        self.check_before_draw();
        self.num_draw_text_call += 1;
        if let Some(mut checker) = self.check_draw_text.pop_front() {
            checker(color, style.family().as_str(), style.size(), pos, text);

            if self.check_draw_text.is_empty() {
                self.check_draw_text.push_back(checker);
            }
        }
        Ok(())
    }

    fn estimate_text_size<TStyle: BackendTextStyle<i32>>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(i32, i32), DrawingErrorKind<Self::ErrorType>> {
        let layout = style
            .layout_box(text)
            .map_err(|e| DrawingErrorKind::FontError(Box::new(e)))?;
        Ok(((layout.1).0 - (layout.0).0, (layout.1).1 - (layout.0).1))
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (w, h) = (self.width, self.height);

        for dx in 0..iw {
            if pos.0 + dx as i32 >= w as i32 {
                break;
            }
            for dy in 0..ih {
                if pos.1 + dy as i32 >= h as i32 {
                    break;
                }
                // FIXME: This assume we have RGB image buffer
                let r = src[(dx + dy * w) as usize * 3];
                let g = src[(dx + dy * w) as usize * 3 + 1];
                let b = src[(dx + dy * w) as usize * 3 + 2];
                let color = BackendColor {
                    alpha: 1.0,
                    rgb: (r, g, b),
                };
                let result = self.draw_pixel((pos.0 + dx as i32, pos.1 + dy as i32), color);
                #[allow(clippy::question_mark)]
                if result.is_err() {
                    return result;
                }
            }
        }

        Ok(())
    }
}

impl Drop for MockedBackend {
    fn drop(&mut self) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.drop_check);

        if let Some(mut checker) = temp {
            checker(self);
        }
    }
}

pub fn create_mocked_drawing_area<F: FnOnce(&mut MockedBackend)>(
    width: u32,
    height: u32,
    setup: F,
) -> DrawingArea<MockedBackend, Shift> {
    let mut backend = MockedBackend::new(width, height);
    setup(&mut backend);
    backend.into_drawing_area()
}
