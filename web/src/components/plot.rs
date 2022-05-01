use anyhow::Result;
use bounce::use_atom_setter;
use gloo_events::EventListener;
use web_sys::HtmlCanvasElement;
use yew::{
    function_component, html, use_effect, use_node_ref, use_state_eq, Callback, MouseEvent,
    Properties,
};

use super::toast::{ToastOption, ToastType};

pub trait Plotter {
    type DataType;

    fn draw_plot(
        &self,
        canvas: HtmlCanvasElement,
        data: &[Self::DataType],
        mouse_coord: Option<(i32, i32)>,
    ) -> Result<()>;
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

    let mouse_coord = use_state_eq(|| -> Option<(i32, i32)> { None });

    {
        let canvas_node_ref = canvas_node_ref.clone();
        let canvas_size = canvas_size.clone();
        let set_toast = set_toast.clone();
        let mouse_coord = mouse_coord.clone();

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
            match plotter.draw_plot(canvas, &data[..], *mouse_coord) {
                Ok(_) => (),
                Err(err) => set_toast(ToastOption::new(
                    format!("Plot drawing error: {err}").to_string(),
                    ToastType::Error,
                )),
            }

            move || drop(listener)
        });
    }

    let onmousemove = {
        let mouse_coord = mouse_coord.clone();
        Callback::from(move |e: MouseEvent| {
            mouse_coord.set(Some((e.offset_x(), e.offset_y())));
        })
    };

    let onmouseleave = {
        let mouse_coord = mouse_coord.clone();
        Callback::from(move |_| {
            mouse_coord.set(None);
        })
    };

    html! {
        <canvas {onmousemove} {onmouseleave} class="fill-space" ref={canvas_node_ref} id="canvas" width={format!("{}", canvas_size.0)} height={format!("{}", canvas_size.1)} />
    }
}
