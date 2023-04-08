/// The dual coordinate system support
use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::mesh::SecondaryMeshStyle;
use super::{ChartContext, ChartState, SeriesAnno};

use crate::coord::cartesian::Cartesian2d;
use crate::coord::ranged1d::{Ranged, ValueFormatter};
use crate::coord::{ReverseCoordTranslate, Shift};

use crate::drawing::DrawingArea;
use crate::drawing::DrawingAreaError;
use crate::element::{Drawable, PointCollection};

use plotters_backend::{BackendCoord, DrawingBackend};

/// The chart context that has two coordinate system attached.
/// This situation is quite common, for example, we with two different coodinate system.
/// For instance this example <img src="https://plotters-rs.github.io/plotters-doc-data/twoscale.png"></img>
/// This is done by attaching  a second coordinate system to ChartContext by method [ChartContext::set_secondary_coord](struct.ChartContext.html#method.set_secondary_coord).
/// For instance of dual coordinate charts, see [this example](https://github.com/38/plotters/blob/master/examples/two-scales.rs#L15).
/// Note: `DualCoordChartContext` is always deref to the chart context.
/// - If you want to configure the secondary axis, method [DualCoordChartContext::configure_secondary_axes](struct.DualCoordChartContext.html#method.configure_secondary_axes)
/// - If you want to draw a series using secondary coordinate system, use [DualCoordChartContext::draw_secondary_series](struct.DualCoordChartContext.html#method.draw_secondary_series). And method [ChartContext::draw_series](struct.ChartContext.html#method.draw_series) will always use primary coordinate spec.
pub struct DualCoordChartContext<'e, CT1, CT2> {
    pub(super) primary: ChartContext<'e, CT1>,
    pub(super) secondary: ChartContext<'e, CT2>,
}

/// The chart state for a dual coord chart, see the detailed description for `ChartState` for more
/// information about the purpose of a chart state.
/// Similar to [ChartState](struct.ChartState.html), but used for the dual coordinate charts.
#[derive(Clone)]
pub struct DualCoordChartState<CT1, CT2> {
    primary: ChartState<CT1>,
    secondary: ChartState<CT2>,
}

impl<CT1, CT2> DualCoordChartContext<'_, CT1, CT2> {
    /// Convert the chart context into a chart state, similar to [ChartContext::into_chart_state](struct.ChartContext.html#method.into_chart_state)
    pub fn into_chart_state(self) -> DualCoordChartState<CT1, CT2> {
        DualCoordChartState {
            primary: self.primary.into(),
            secondary: self.secondary.into(),
        }
    }

    /// Convert the chart context into a sharable chart state.
    pub fn into_shared_chart_state(self) -> DualCoordChartState<Arc<CT1>, Arc<CT2>> {
        DualCoordChartState {
            primary: self.primary.into_shared_chart_state(),
            secondary: self.secondary.into_shared_chart_state(),
        }
    }

    /// Copy the coordinate specs and make a chart state
    pub fn to_chart_state(&self) -> DualCoordChartState<CT1, CT2>
    where
        CT1: Clone,
        CT2: Clone,
    {
        DualCoordChartState {
            primary: self.primary.to_chart_state(),
            secondary: self.secondary.to_chart_state(),
        }
    }
}

impl<CT1, CT2> DualCoordChartState<CT1, CT2> {
    /// Restore the chart state on the given drawing area
    pub fn restore<'e>(self, area: &DrawingArea<Shift>) -> DualCoordChartContext<'e, CT1, CT2> {
        let primary = self.primary.restore(area);
        let secondary = self
            .secondary
            .restore(&primary.plotting_area().strip_coord_spec());
        DualCoordChartContext { primary, secondary }
    }
}

impl<CT1, CT2> From<DualCoordChartContext<'_, CT1, CT2>> for DualCoordChartState<CT1, CT2> {
    fn from(chart: DualCoordChartContext<'_, CT1, CT2>) -> DualCoordChartState<CT1, CT2> {
        chart.into_chart_state()
    }
}

impl<'b, CT1: Clone, CT2: Clone> From<&'b DualCoordChartContext<'_, CT1, CT2>>
    for DualCoordChartState<CT1, CT2>
{
    fn from(chart: &'b DualCoordChartContext<'_, CT1, CT2>) -> DualCoordChartState<CT1, CT2> {
        chart.to_chart_state()
    }
}

impl<'e, CT1, CT2> DualCoordChartContext<'e, CT1, CT2> {
    pub(super) fn new(mut primary: ChartContext<'e, CT1>, secondary_coord: CT2) -> Self {
        let secondary_drawing_area = primary
            .drawing_area
            .strip_coord_spec()
            .apply_coord_spec(secondary_coord);
        let mut secondary_x_label_area = [None, None];
        let mut secondary_y_label_area = [None, None];

        std::mem::swap(&mut primary.x_label_area[0], &mut secondary_x_label_area[0]);
        std::mem::swap(&mut primary.y_label_area[1], &mut secondary_y_label_area[1]);

        Self {
            primary,
            secondary: ChartContext {
                x_label_area: secondary_x_label_area,
                y_label_area: secondary_y_label_area,
                drawing_area: secondary_drawing_area,
                series_anno: vec![],
                drawing_area_pos: (0, 0),
            },
        }
    }

    /// Get a reference to the drawing area that uses the secondary coordinate system
    pub fn secondary_plotting_area(&self) -> &DrawingArea<CT2> {
        &self.secondary.drawing_area
    }

    /// Borrow a mutable reference to the chart context that uses the secondary
    /// coordinate system
    pub fn borrow_secondary(&self) -> &ChartContext<'e, CT2> {
        &self.secondary
    }
}

impl<CT1, CT2: ReverseCoordTranslate> DualCoordChartContext<'_, CT1, CT2> {
    /// Convert the chart context into the secondary coordinate translation function
    pub fn into_secondary_coord_trans(self) -> impl Fn(BackendCoord) -> Option<CT2::From> {
        let coord_spec = self.secondary.drawing_area.into_coord_spec();
        move |coord| coord_spec.reverse_translate(coord)
    }
}

impl<CT1: ReverseCoordTranslate, CT2: ReverseCoordTranslate> DualCoordChartContext<'_, CT1, CT2> {
    /// Convert the chart context into a pair of closures that maps the pixel coordinate into the
    /// logical coordinate for both primary coordinate system and secondary coordinate system.
    pub fn into_coord_trans_pair(
        self,
    ) -> (
        impl Fn(BackendCoord) -> Option<CT1::From>,
        impl Fn(BackendCoord) -> Option<CT2::From>,
    ) {
        let coord_spec_1 = self.primary.drawing_area.into_coord_spec();
        let coord_spec_2 = self.secondary.drawing_area.into_coord_spec();
        (
            move |coord| coord_spec_1.reverse_translate(coord),
            move |coord| coord_spec_2.reverse_translate(coord),
        )
    }
}

impl<'e, CT1, XT, YT, SX: Ranged<ValueType = XT>, SY: Ranged<ValueType = YT>>
    DualCoordChartContext<'e, CT1, Cartesian2d<SX, SY>>
where
    SX: ValueFormatter<XT>,
    SY: ValueFormatter<YT>,
{
    /// Start configure the style for the secondary axes
    pub fn configure_secondary_axes<'b>(&'b mut self) -> SecondaryMeshStyle<'b, 'e, SX, SY> {
        SecondaryMeshStyle::new(&mut self.secondary)
    }
}

impl<'e, X: Ranged, Y: Ranged, SX: Ranged, SY: Ranged>
    DualCoordChartContext<'e, Cartesian2d<X, Y>, Cartesian2d<SX, SY>>
{
    /// Draw a series use the secondary coordinate system.
    /// - `series`: The series to draw
    /// - `Returns` the series annotation object or error code
    pub fn draw_secondary_series<DB, E, R, S>(
        &mut self,
        backend: &mut DB,
        series: S,
    ) -> Result<&mut SeriesAnno<'e>, DrawingAreaError>
    where
        DB: DrawingBackend,
        for<'b> &'b E: PointCollection<'b, (SX::ValueType, SY::ValueType)>,
        E: Drawable,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        self.secondary.draw_series_impl(backend, series)?;
        Ok(self.primary.alloc_series_anno())
    }
}

impl<'e, CT1, CT2> Borrow<ChartContext<'e, CT1>> for DualCoordChartContext<'e, CT1, CT2> {
    fn borrow(&self) -> &ChartContext<'e, CT1> {
        &self.primary
    }
}

impl<'e, CT1, CT2> BorrowMut<ChartContext<'e, CT1>> for DualCoordChartContext<'e, CT1, CT2> {
    fn borrow_mut(&mut self) -> &mut ChartContext<'e, CT1> {
        &mut self.primary
    }
}

impl<'e, CT1, CT2> Deref for DualCoordChartContext<'e, CT1, CT2> {
    type Target = ChartContext<'e, CT1>;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl<'e, CT1, CT2> DerefMut for DualCoordChartContext<'e, CT1, CT2> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}
