use std::cmp::Ordering::Equal;

use anyhow::{anyhow, Result};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;

use crate::components::plot::Plotter;

#[derive(Clone, PartialEq)]
pub struct EntropiesPlotter;

impl Plotter for EntropiesPlotter {
    type DataType = f64;

    fn draw_plot(&self, canvas: HtmlCanvasElement, data: &[f64]) -> Result<()> {
        let root = CanvasBackend::with_canvas_object(canvas)
            .ok_or(anyhow!("Unable to initialize plot backend from canvas"))?
            .into_drawing_area();

        let mut data = data.iter().copied().collect::<Vec<_>>();
        data.sort_by(|v1, v2| v2.partial_cmp(v1).unwrap_or(Equal));

        root.fill(&WHITE)?;

        let y_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let y_max = if data.len() > 0 { y_max + 0.02 } else { 1.0 };

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35u32)
            .y_label_area_size(60u32)
            .margin(8u32)
            .build_cartesian_2d((0..data.len()).into_segmented(), 0.0..y_max)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .disable_x_axis()
            .y_desc("probability")
            .x_desc("hints")
            .axis_desc_style(("sans-serif", 15u32))
            .draw()?;

        chart.draw_series(data.into_iter().enumerate().map(|(x, y)| {
            let x0 = SegmentValue::Exact(x);
            let x1 = SegmentValue::Exact(x + 1);
            Rectangle::new([(x0, 0.), (x1, y)], BLUE.filled())
        }))?;

        root.present().expect("Unable to draw");

        Ok(())
    }
}
