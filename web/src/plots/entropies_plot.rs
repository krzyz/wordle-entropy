use std::{cmp::Ordering::Equal, iter, rc::Rc};

use anyhow::{anyhow, Result};
use plotters::{coord::ReverseCoordTranslate, prelude::*};
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;
use wordle_entropy_core::structs::hints::Hint;

use crate::{components::Plotter, word_set::WordSet, EntropiesData};

#[derive(Clone, PartialEq)]
pub struct EntropiesPlotter {
    pub word_set: Rc<WordSet>,
    pub entropies_data: Option<EntropiesData>,
}

impl Plotter for EntropiesPlotter {
    type DataType = f64;

    fn draw_plot(
        &self,
        canvas: HtmlCanvasElement,
        data: &[f64],
        mouse_coord: Option<(i32, i32)>,
    ) -> Result<()> {
        let root = CanvasBackend::with_canvas_object(canvas)
            .ok_or(anyhow!("Unable to initialize plot backend from canvas"))?
            .into_drawing_area();

        let mut data_indexed = data.iter().copied().enumerate().collect::<Vec<_>>();
        data_indexed.sort_by(|(_, v1), (_, v2)| v2.partial_cmp(v1).unwrap_or(Equal));
        let (indices, data): (Vec<usize>, Vec<f64>) = data_indexed.into_iter().unzip();

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

        let xy = if data.len() > 0 {
            if let Some(mouse_coord) = mouse_coord {
                let coord_spec = chart.as_coord_spec();
                let plot_coord = coord_spec.reverse_translate(mouse_coord);

                if let Some((SegmentValue::Exact(x), y)) | Some((SegmentValue::CenterOf(x), y)) =
                    plot_coord
                {
                    Some((x, y))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        chart.draw_series(data.into_iter().enumerate().map(|(x, y)| {
            let x0 = SegmentValue::Exact(x);
            let x1 = SegmentValue::Exact(x + 1);
            let style = if xy.filter(|&(selected_x, _)| selected_x == x).is_some() {
                RED
            } else {
                BLUE
            };

            Rectangle::new([(x0, 0.), (x1, y)], style.filled())
        }))?;

        if let Some((x, y)) = xy {
            if let Some((hints, prob)) = indices.get(x).and_then(|&index| {
                self.word_set.dictionary.hints.get(index).and_then(|hints| {
                    self.entropies_data.as_ref().and_then(|entropies_data| {
                        entropies_data
                            .probabilities
                            .get(index)
                            .map(|&prob| (hints, prob))
                    })
                })
            }) {
                let size = 25;
                let padded_size = size + 3;

                chart.draw_series(iter::once(
                    EmptyElement::at((SegmentValue::Exact(x), y))
                        + Rectangle::new(
                            [
                                (0, -size - 10),
                                (15 + padded_size * hints.0.len() as i32 + 5, 25),
                            ],
                            WHITE.filled(),
                        ),
                ))?;

                for (i, h) in hints.0.into_iter().enumerate() {
                    let style = match h {
                        Hint::Wrong => RGBColor(58, 58, 60),
                        Hint::OutOfPlace => RGBColor(181, 159, 59),
                        Hint::Correct => RGBColor(83, 141, 78),
                    };

                    let i = i as i32;

                    chart.draw_series(iter::once(
                        EmptyElement::at((SegmentValue::Exact(x), y))
                            + Rectangle::new(
                                [
                                    (5 + padded_size * i, -size - 5),
                                    (5 + padded_size * i + size, -5),
                                ],
                                style.filled(),
                            ),
                    ))?;
                }

                chart.draw_series(iter::once(
                    EmptyElement::at((SegmentValue::Exact(x), y))
                        + Text::new(format!("p = {prob:.3}"), (25, 5), ("sans-serif", 15)),
                ))?;
            }
        }

        root.present().expect("Unable to draw");

        Ok(())
    }
}
