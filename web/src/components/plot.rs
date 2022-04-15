use anyhow::Result;
use bounce::use_atom_setter;
use gloo_events::EventListener;
use web_sys::HtmlCanvasElement;
use yew::{function_component, html, use_effect, use_node_ref, use_state_eq, Properties};

use super::toast::{ToastOption, ToastType};

pub trait Plotter {
    type DataType;

    fn draw_plot(&self, canvas: HtmlCanvasElement, data: &[Self::DataType]) -> Result<()>;
}

#[derive(Properties, PartialEq)]
pub struct Props<T, P>
where
    T: PartialEq,
    P: PartialEq + Plotter<DataType = T>,
{
    pub data: Vec<T>,
    pub plotter: P,
}

#[function_component(Plot)]
pub fn view<T, P>(props: &Props<T, P>) -> Html
where
    T: Clone + PartialEq + 'static,
    P: Clone + PartialEq + Plotter<DataType = T> + 'static,
{
    let data = props.data.clone();
    let plotter = props.plotter.clone();
    let canvas_node_ref = use_node_ref();
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
            match plotter.draw_plot(canvas, &data[..]) {
                Ok(_) => (),
                Err(err) => set_toast(ToastOption::new(
                    format!("Plot drawing error: {err}").to_string(),
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
