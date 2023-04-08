use std::sync::Arc;

use super::ChartContext;
use crate::coord::Shift;
use crate::drawing::DrawingArea;

/// A chart context state - This is the data that is needed to reconstruct the chart context
/// without actually drawing the chart. This is useful when we want to do realtime rendering and
/// want to incrementally update the chart.
///
/// For each frame, instead of updating the entire backend, we are able to keep the keep the figure
/// component like axis, labels untouched and make updates only in the plotting drawing area.
/// This is very useful for incremental render.
/// ```rust
///     use plotters::prelude::*;
///     let mut buffer = vec![0u8;1024*768*3];
///     let mut backend = BitMapBackend::with_buffer(&mut buffer[..], (1024, 768));
///     let area = backend
///         .to_drawing_area()
///         .split_evenly((1,2));
///     let chart = ChartBuilder::on(&area[0])
///         .caption("Incremental Example", ("sans-serif", 20))
///         .set_all_label_area_size(30)
///         .build_cartesian_2d(&mut backend, 0..10, 0..10)
///         .expect("Unable to build ChartContext");
///     // Draw the first frame at this point
///     area[0].present(&mut backend).expect("Present");
///     let state = chart.into_chart_state();
///     // Let's draw the second frame
///     let chart = state.restore(&area[0]);
///     chart.plotting_area().fill(&mut backend, &WHITE).unwrap(); // Clear the previously drawn graph
///     // At this point, you are able to draw next frame
/// ```
#[derive(Clone)]
pub struct ChartState<CT> {
    drawing_area_pos: (i32, i32),
    drawing_area_size: (u32, u32),
    coord: CT,
}

impl<'e, CT> From<ChartContext<'e, CT>> for ChartState<CT> {
    fn from(chart: ChartContext<'e, CT>) -> ChartState<CT> {
        ChartState {
            drawing_area_pos: chart.drawing_area_pos,
            drawing_area_size: chart.drawing_area.dim_in_pixel(),
            coord: chart.drawing_area.into_coord_spec(),
        }
    }
}

impl<'e, CT> ChartContext<'e, CT> {
    /// Convert a chart context into a chart state, by doing so, the chart context is consumed and
    /// a saved chart state is created for later use. This is typically used in incrmental rendering. See documentation of `ChartState` for more detailed example.
    pub fn into_chart_state(self) -> ChartState<CT> {
        self.into()
    }

    /// Convert the chart context into a sharable chart state.
    /// Normally a chart state can not be clone, since the coordinate spec may not be able to be
    /// cloned. In this case, we can use an `Arc` get the coordinate wrapped thus the state can be
    /// cloned and shared by multiple chart context
    pub fn into_shared_chart_state(self) -> ChartState<Arc<CT>> {
        ChartState {
            drawing_area_pos: self.drawing_area_pos,
            drawing_area_size: self.drawing_area.dim_in_pixel(),
            coord: Arc::new(self.drawing_area.into_coord_spec()),
        }
    }
}

impl<'e, CT: Clone> From<&ChartContext<'e, CT>> for ChartState<CT> {
    fn from(chart: &ChartContext<'e, CT>) -> ChartState<CT> {
        ChartState {
            drawing_area_pos: chart.drawing_area_pos,
            drawing_area_size: chart.drawing_area.dim_in_pixel(),
            coord: chart.drawing_area.as_coord_spec().clone(),
        }
    }
}

impl<'e, CT: Clone> ChartContext<'e, CT> {
    /// Make the chart context, do not consume the chart context and clone the coordinate spec
    pub fn to_chart_state(&self) -> ChartState<CT> {
        self.into()
    }
}

impl<CT> ChartState<CT> {
    /// Restore the chart context on the given drawing area
    ///
    /// - `area`: The given drawing area where we want to restore the chart context
    /// - **returns** The newly created chart context
    pub fn restore<'e>(self, area: &DrawingArea<Shift>) -> ChartContext<'e, CT> {
        let area = area
            .clone()
            .shrink(self.drawing_area_pos, self.drawing_area_size);
        ChartContext {
            x_label_area: [None, None],
            y_label_area: [None, None],
            drawing_area: area.apply_coord_spec(self.coord),
            series_anno: vec![],
            drawing_area_pos: self.drawing_area_pos,
        }
    }
}
