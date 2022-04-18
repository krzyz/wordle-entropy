use anyhow::{anyhow, Result};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;
use wordle_entropy_core::calibration::{bounded_log_c, fit, Calibration};

use crate::components::Plotter;

#[derive(Clone, PartialEq)]
pub struct TurnsLeftPlotter;

impl Plotter for TurnsLeftPlotter {
    type DataType = (f64, f64, f64);

    fn draw_plot(
        &self,
        canvas: HtmlCanvasElement,
        data_with_weights: &[(f64, f64, f64)],
    ) -> Result<()> {
        let root = CanvasBackend::with_canvas_object(canvas)
            .ok_or(anyhow!("Unable to initialize plot backend from canvas"))?
            .into_drawing_area();

        let data = data_with_weights
            .into_iter()
            .copied()
            .map(|(c1, c2, _)| (c1, c2))
            .collect::<Vec<_>>();

        let y_max = 1.
            + data_with_weights
                .iter()
                .filter(|&&(_, _, prob)| prob > 0.2)
                .map(|&(_, left, _)| left)
                .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Less))
                .unwrap_or(7.) as f64;

        let x_max = 1.
            + data
                .iter()
                .map(|&(entropy, _)| entropy)
                .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Less))
                .unwrap_or(5.) as f64;

        let x_max_i = x_max.ceil() as i32;

        let bar_per_1 = 4;
        let mut bars = (0..(bar_per_1 * x_max_i))
            .map(|i| {
                (
                    (
                        i as f64 / bar_per_1 as f64,
                        (i + 1) as f64 / bar_per_1 as f64,
                    ),
                    Vec::<(f64, f64)>::new(),
                )
            })
            .collect::<Vec<_>>();

        for &(x, y, prob) in data_with_weights.iter() {
            if let Some((_, bar_vec)) = bars.get_mut((x * bar_per_1 as f64).floor() as usize) {
                bar_vec.push((y, prob));
            }
        }

        let bars = bars
            .into_iter()
            .filter(|(_, bar_vec)| bar_vec.len() > 0)
            .collect::<Vec<_>>();

        let weights = bars.iter().map(|_| 1.).collect::<Vec<f64>>();

        let bar_data = bars
            .into_iter()
            .map(|(x, bar_vec)| {
                let norm: f64 = bar_vec.iter().map(|&(_, prob)| prob).sum();
                (
                    x,
                    bar_vec.into_iter().map(|(y, prob)| prob * y).sum::<f64>() / norm,
                )
            })
            .collect::<Vec<_>>();

        root.fill(&WHITE)?;
        let root = root.margin(10u32, 10u32, 10u32, 10u32);
        let mut chart = ChartBuilder::on(&root)
            .margin(5u32)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(0f64..x_max, 0f64..y_max)?
            .set_secondary_coord(
                (0f64..x_max)
                    .step(1. / bar_per_1 as f64)
                    .use_round()
                    .into_segmented(),
                0f64..y_max,
            );

        chart.configure_mesh().draw()?;

        chart.configure_secondary_axes().draw()?;

        chart.draw_secondary_series(bar_data.iter().copied().map(|((x0, x1), y)| {
            let x0 = SegmentValue::Exact(x0);
            let x1 = SegmentValue::Exact(x1);
            Rectangle::new([(x0, 0.), (x1, y)], RED.mix(0.2).filled())
        }))?;

        let fit_data = bar_data
            .into_iter()
            .map(|((x0, x1), y)| (0.5 * (x0 + x1), y))
            .collect::<Vec<_>>();

        let axis_val_multiplier = 5.;
        if fit_data.len() >= 4 {
            let calibration = fit(fit_data, weights).map_err(|e| anyhow!("{e}"))?;
            let color = &RED;
            chart
                .draw_series(LineSeries::new(
                    (0..=((axis_val_multiplier * x_max.floor()) as i32))
                        .map(|x| (x as f64) / axis_val_multiplier)
                        .map(|x| (x, bounded_log_c(x, calibration))),
                    color,
                ))?
                .label(format!("Fitted calibration"))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        {
            let color = &BLUE;
            chart
                .draw_series(LineSeries::new(
                    (0..=((axis_val_multiplier * x_max.floor()) as i32))
                        .map(|x| (x as f64) / axis_val_multiplier)
                        .map(|x| (x, bounded_log_c(x, Calibration::default()))),
                    color,
                ))?
                .label(format!("Used calibration"))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        let circle_size = 2;
        let circle_color = &BLACK;
        chart
            .draw_series(PointSeries::of_element(
                data_with_weights.iter().copied(),
                circle_size,
                circle_color,
                &|(c1, c2, prob), s: i32, st| {
                    return Circle::new((c1, c2), s, {
                        let mut st = st.filled();
                        st.color = BLACK.mix(prob);
                        st
                    });
                },
            ))?
            .label(format!("Known data points"))
            .legend(move |(x, y)| Circle::new((x, y), circle_size, circle_color));

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(root.present().map_err(anyhow::Error::msg)?)
    }
}