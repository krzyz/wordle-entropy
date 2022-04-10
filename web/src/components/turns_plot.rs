use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};
use web_sys::HtmlCanvasElement;
use yew::{functional::function_component, html, use_effect, use_node_ref, Properties};

pub fn bounded_log<S: Scalar + Float>(x: S, a1: S, a2: S, a3: S) -> S {
    let val = a1 + a2 * (x + a3).ln();
    if val > One::one() {
        val
    } else {
        One::one()
    }
}

pub fn bounded_log_v<S: Scalar + Float>(x: &DVector<S>, a1: S, a2: S, a3: S) -> DVector<S> {
    x.map(|x| bounded_log(x, a1, a2, a3))
}

pub fn bounded_log_da1<S: Scalar + Float>(x: &DVector<S>, a1: S, a2: S, a3: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a1, a2, a3) > one {
            one - one
        } else {
            one
        }
    })
}

pub fn bounded_log_da2<S: Scalar + Float>(x: &DVector<S>, a1: S, a2: S, a3: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a1, a2, a3) > one {
            (x + a3).ln()
        } else {
            one
        }
    })
}

pub fn bounded_log_da3<S: Scalar + Float>(x: &DVector<S>, a1: S, a2: S, a3: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a1, a2, a3) > one {
            a2 / (x + a3)
        } else {
            one
        }
    })
}

fn fit(data: Vec<(f64, f64)>) -> (f64, f64, f64, f64) {
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    let model = SeparableModelBuilder::<f64>::new(&["a1", "a2", "a3"])
        .function(&["a1", "a2", "a3"], bounded_log_v)
        .partial_deriv("a1", bounded_log_da1)
        .partial_deriv("a2", bounded_log_da2)
        .partial_deriv("a3", bounded_log_da3)
        .build()
        .unwrap();

    let problem = LevMarProblemBuilder::new()
        .model(&model)
        .x(x)
        .y(y)
        .initial_guess(&[-2., 3., 1.])
        .build()
        .unwrap();

    let (solved_problem, report) = LevMarSolver::new().minimize(problem);
    assert!(report.termination.was_successful());
    let alpha = solved_problem.params();
    let c = solved_problem.linear_coefficients().unwrap();

    (c[0], alpha[0], alpha[1], alpha[2])
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

    let axis_val_multiplier = 5.;
    if data.len() >= 4 {
        let (c, a1, a2, a3) = fit(data.iter().cloned().collect());
        chart.draw_series(LineSeries::new(
            (0..=((axis_val_multiplier * x_max.floor()) as i32))
                .map(|x| (x as f64) / axis_val_multiplier)
                .map(|x| (x, c * bounded_log(x, a1, a2, a3))),
            &RED,
        ))?;
    }

    {
        let (c, a1, a2, a3) = (1., -2., 3., 1.);
        chart.draw_series(LineSeries::new(
            (0..=((axis_val_multiplier * x_max.floor()) as i32))
                .map(|x| (x as f64) / axis_val_multiplier)
                .map(|x| (x, c * bounded_log(x, a1, a2, a3))),
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
