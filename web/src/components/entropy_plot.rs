use std::cmp::Ordering::Equal;

use anyhow::{anyhow, Result};
use bounce::use_atom_setter;
use gloo_events::EventListener;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;
use yew::{
    functional::function_component, html, use_effect, use_node_ref, use_state_eq, Properties,
};

use super::toast::{ToastOption, ToastType};

fn draw_plot(canvas: HtmlCanvasElement, data: &[f64]) -> Result<()> {
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

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<f64>,
}

#[function_component(EntropyPlot)]
pub fn view(props: &Props) -> Html {
    let canvas_node_ref = use_node_ref();
    let data = props.data.clone();
    let canvas_size = use_state_eq(|| (700., 400.));
    let set_toast = use_atom_setter::<ToastOption>();

    {
        let canvas_node_ref = canvas_node_ref.clone();
        let canvas_size = canvas_size.clone();
        let set_toast = set_toast.clone();

        use_effect(move || {
            let listener = {
                let canvas_node_ref = canvas_node_ref.clone();
                let canvas_size = canvas_size.clone();
                EventListener::new(&gloo_utils::window(), "resize", move |_| {
                    let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
                    let dom_rect = canvas.get_bounding_client_rect();
                    canvas_size.set((dom_rect.width(), dom_rect.height()));
                })
            };

            let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
            let dom_rect = canvas.get_bounding_client_rect();
            canvas_size.set((dom_rect.width(), dom_rect.height()));
            match draw_plot(canvas, &data[..]) {
                Ok(_) => (),
                Err(err) => set_toast(ToastOption::new(
                    format!("Worker error: {err}").to_string(),
                    ToastType::Error,
                )),
            }

            move || drop(listener)
        });
    }

    html! {
        <canvas class="fill-space" ref={canvas_node_ref} id="canvas" width={format!("{}", canvas_size.0)} height={format!("{}", canvas_size.1)} />
    }
}
