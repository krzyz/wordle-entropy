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

    root.fill(&WHITE)?;
    let root = root.margin(10u32, 10u32, 10u32, 10u32);
    let mut chart = ChartBuilder::on(&root)
        .margin(5u32)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(0f64..x_max, 0f64..y_max)?;

    chart.configure_mesh().draw()?;

    let axis_val_multiplier = 5.;
    if data.len() >= 4 {
        let calibration = fit(data.iter().copied().collect(), weights);
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
