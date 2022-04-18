use yew::{function_component, html, Properties};

use crate::{components::Plot, plots::TurnsLeftPlotter};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<(f64, f64, f64)>,
}

#[function_component(Calibration)]
pub fn view(props: &Props) -> Html {
    let plotter = TurnsLeftPlotter;
    html! {
        <Plot<(f64, f64, f64), TurnsLeftPlotter> data={props.data.clone()} {plotter} />
    }
}
