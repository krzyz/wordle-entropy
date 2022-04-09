use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};
use web_sys::HtmlCanvasElement;
use yew::{functional::function_component, html, use_effect, use_node_ref, Properties};

// c * (x+1)^r log((x+1))
pub fn log_f_s<S: Scalar + Float>(x: S, r: S, a: S, b: S) -> S {
    let x = x + One::one();
    b + a * x.powf(r) * x.ln()
}

pub fn log_f<S: Scalar + Float>(x: &DVector<S>, r: S) -> DVector<S> {
    x.map(|x| x + One::one()).map(|x| x.powf(r) * x.ln())
}

pub fn log_f_dr<S: Scalar + Float>(rvec: &DVector<S>, r: S) -> DVector<S> {
    rvec.map(|x| x + One::one())
        .map(|x| r * x.powf(r - One::one()) * x.ln())
}

fn fit(data: Vec<(f64, f64)>) -> (f64, f64, f64) {
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    let model = SeparableModelBuilder::<f64>::new(&["r"])
        .function(&["r"], log_f)
        .partial_deriv("r", log_f_dr)
        .invariant_function(|x| x.clone())
        .build()
        .unwrap();

    let problem = LevMarProblemBuilder::new()
        .model(&model)
        .x(x)
        .y(y)
        .initial_guess(&[0.])
        .build()
        .unwrap();

    let (solved_problem, report) = LevMarSolver::new().minimize(problem);
    assert!(report.termination.was_successful());
    let alpha = solved_problem.params();
    let c = solved_problem.linear_coefficients().unwrap();

    (c[0], c[1], alpha[0])
}

fn draw_plot(
    canvas: HtmlCanvasElement,
    data: &[(f64, f64)],
    words_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

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

    if data.len() >= 3 {
        let (a, b, r) = fit(data.iter().cloned().collect());
        let c = 5.;
        chart.draw_series(LineSeries::new(
            (0..=((c * x_max.floor()) as i32))
                .map(|x| (x as f64) / c)
                .map(|x| (x, log_f_s(x, r, a, b))),
            &RED,
        ))?;
    }

    {
        let (a, b, r) = (1.6369421, -0.029045254, 0.);
        let c = 5.;
        chart.draw_series(LineSeries::new(
            (0..=((c * x_max.floor()) as i32))
                .map(|x| (x as f64) / c)
                .map(|x| (x, log_f_s(x, r, a, b))),
            &BLUE,
        ))?;
    }

    chart.draw_series(PointSeries::of_element(
        data.iter().copied(),
        2,
        &BLACK,
        &|c, s: i32, st| {
            return Circle::new(c, s, st.filled());
        },
    ))?;

    root.present().expect("Unable to draw");

    Ok(())
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub data: Vec<(f64, f64)>,
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
