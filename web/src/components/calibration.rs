use bounce::use_slice_dispatch;
use wordle_entropy_core::calibration::{fit, Calibration as CalibrationParameters};
use yew::{function_component, html, Callback, Properties};

use crate::{
    components::Plot,
    plots::TurnsLeftPlotter,
    word_set::{SetCalibration, WordSetVec, WordSetVecAction},
};

fn calibrate(bar_data: &Vec<((f64, f64), f64)>) -> Option<CalibrationParameters> {
    let fit_data = bar_data
        .into_iter()
        .map(|&((x0, x1), y)| (0.5 * (x0 + x1), y))
        .collect::<Vec<_>>();

    fit(fit_data, None).ok()
}

fn calc_bar_data(
    data_with_weights: &Vec<(f64, f64, f64)>,
    bar_per_1: i32,
) -> Vec<((f64, f64), f64)> {
    let data = data_with_weights
        .into_iter()
        .copied()
        .map(|(c1, c2, _)| (c1, c2))
        .collect::<Vec<_>>();

    let x_max = 1.
        + data
            .iter()
            .map(|&(entropy, _)| entropy)
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Less))
            .unwrap_or(5.) as f64;

    let x_max_i = x_max.ceil() as i32;

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

    bar_data
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<(f64, f64, f64)>,
    pub word_set_name: String,
    pub used_calibration: CalibrationParameters,
}

#[function_component(Calibration)]
pub fn view(props: &Props) -> Html {
    let dispatch_word_sets = use_slice_dispatch::<WordSetVec>();
    let bar_per_1 = 4;
    let bar_data = calc_bar_data(&props.data, bar_per_1);
    let calibration = calibrate(&bar_data);

    let on_set_calibrate_click = {
        let word_set_name = props.word_set_name.clone();
        Callback::from(move |_| {
            if let Some(calibration) = calibration {
                dispatch_word_sets(WordSetVecAction::SetCalibration(
                    word_set_name.clone(),
                    SetCalibration::Custom(calibration),
                ))
            }
        })
    };

    let plotter = TurnsLeftPlotter {
        calibration,
        used_calibration: props.used_calibration,
        bar_per_1,
        bar_data,
    };

    html! {
        <>
            <Plot<(f64, f64, f64), TurnsLeftPlotter> data={props.data.clone()} {plotter} />
            <button
                    class="btn btn-primary"
                    onclick={on_set_calibrate_click}
            >{ "Set current calibration" }</button>
        </>
    }
}
