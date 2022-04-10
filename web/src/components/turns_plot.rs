use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;
use wordle_entropy_core::calibration::{bounded_log_c, fit, Calibration};
use yew::{functional::function_component, html, use_effect, use_node_ref, Properties};

fn draw_plot(
    canvas: HtmlCanvasElement,
    data_with_weights: &[(f64, f64, f64)],
    words_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

    let data = data_with_weights
        .into_iter()
        .copied()
        .map(|(c1, c2, _)| (c1, c2))
        .collect::<Vec<_>>();

    let weights = data_with_weights
        .into_iter()
        .copied()
        .map(|(_, _, c3)| c3)
        .collect::<Vec<_>>();

    let y_max = 1.
        + data
            .iter()
            .map(|&(_, left)| left)
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap_or(7.) as f64;
    let x_max = (words_len as f64).log2() + 1.;

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

    let bar_data = bars
        .into_iter()
        .filter(|(_, bar_vec)| bar_vec.len() > 0)
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
    if data.len() >= 4 {
        let calibration = fit(fit_data);
        chart
            .draw_series(LineSeries::new(
                (0..=((axis_val_multiplier * x_max.floor()) as i32))
                    .map(|x| (x as f64) / axis_val_multiplier)
                    .map(|x| (x, bounded_log_c(x, calibration))),
                &RED,
            ))?
            .label(format!("{calibration:#?}"))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }

    {
        chart.draw_series(LineSeries::new(
            (0..=((axis_val_multiplier * x_max.floor()) as i32))
                .map(|x| (x as f64) / axis_val_multiplier)
                .map(|x| (x, bounded_log_c(x, Calibration::default()))),
            &BLUE,
        ))?;
    }

    chart.draw_series(PointSeries::of_element(
        data_with_weights.iter().copied(),
        2,
        &BLACK,
        &|(c1, c2, prob), s: i32, st| {
            return Circle::new((c1, c2), s, {
                let mut st = st.filled();
                st.color = BLACK.mix(prob);
                st
            });
        },
    ))?;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present().expect("Unable to draw");

    Ok(())
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<(f64, f64, f64)>,
    pub words_len: usize,
}

#[function_component(TurnsPlot)]
pub fn view(props: &Props) -> Html {
    let canvas_node_ref = use_node_ref();
    let data = props.data.clone();
    let words_len = props.words_len;

    {
        let canvas_node_ref = canvas_node_ref.clone();
        use_effect(move || {
            let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
            draw_plot(canvas, &data[..], words_len).unwrap();
            || ()
        });
    }

    html! {
        <canvas ref={canvas_node_ref} id="canvas" width="800" height="400" />
    }
}
