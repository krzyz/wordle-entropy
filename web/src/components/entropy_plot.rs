use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::cmp::Ordering::Equal;
use web_sys::HtmlCanvasElement;
use yew::{functional::function_component, html, use_node_ref, Properties, use_effect};

fn draw_plot(canvas: HtmlCanvasElement, data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

    let mut data = data.iter().copied().collect::<Vec<_>>();
    data.sort_by(|v1, v2| v2.partial_cmp(v1).unwrap_or(Equal));

    root.fill(&WHITE)?;

    let y_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_max = if data.len() > 0 { y_max + 0.02 } else { 1.0 };

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .margin(8u32)
        .build_cartesian_2d((0..data.len()).into_segmented(), 0.0..y_max)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .disable_x_axis()
        .y_desc("probability")
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

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<f64>
}

#[function_component(EntropyPlot)]
pub fn view(props: &Props) -> Html {
    let canvas_node_ref = use_node_ref();
    let data = props.data.clone();

    {
        let canvas_node_ref = canvas_node_ref.clone();
        use_effect(move || {
            let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
            draw_plot(canvas, &data[..]).unwrap();
            || ()
        });
    }

    html! {
        <canvas ref={canvas_node_ref} id="canvas" width="800" height="400" />
    }
}
