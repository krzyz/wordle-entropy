use anyhow::{anyhow, Result};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;
use wordle_entropy_core::FxHashMap;

use crate::components::plot::Plotter;

#[derive(Clone, PartialEq)]
pub struct ExpectedTurnsPlotter {
    pub weighted: bool,
}

impl Plotter for ExpectedTurnsPlotter {
    type DataType = (usize, f64);

    fn draw_plot(&self, canvas: HtmlCanvasElement, data: &[(usize, f64)]) -> Result<()> {
        let root = CanvasBackend::with_canvas_object(canvas)
            .ok_or(anyhow!("Unable to initialize plot backend from canvas"))?
            .into_drawing_area();

        let mut counts: FxHashMap<usize, f64> = FxHashMap::default();
        for &(turns_left, prob) in data.iter() {
            let to_add = if self.weighted { prob } else { 1. };
            *counts.entry(turns_left).or_insert(0.) += to_add;
        }
        let bars = counts.into_iter().collect::<Vec<_>>();

        root.fill(&WHITE)?;

        let x_max = bars.iter().map(|&(x, _)| x).max().unwrap_or(6);

        let y_max = 1.
            + bars
                .iter()
                .map(|&(_, y)| y)
                .fold(f64::NEG_INFINITY, f64::max);
        let y_max = if bars.len() > 0 { y_max + 0.02 } else { 1.0 };

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35u32)
            .y_label_area_size(60u32)
            .margin(8u32)
            .build_cartesian_2d((0..x_max).into_segmented(), 0.0..y_max)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc("# of occurencies × probability")
            .x_desc("turns")
            .axis_desc_style(("sans-serif", 15u32))
            .draw()?;

        chart.draw_series(bars.into_iter().map(|(x, y)| {
            let x0 = SegmentValue::Exact(x);
            let x1 = SegmentValue::Exact(x + 1);
            let mut bar = Rectangle::new([(x0, 0.), (x1, y)], BLUE.filled());
            bar.set_margin(0, 0, 5, 5);
            bar
        }))?;

        root.present().expect("Unable to draw");

        Ok(())
    }
}
