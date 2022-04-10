use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

#[derive(Copy, Clone, Debug)]
pub struct Calibration {
    c: f64,
    a0: f64,
    a1: f64,
    a2: f64,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            c: 0.2372,
            a0: -2.1664,
            a1: 10.209,
            a2: 2.4787,
        }
    }
}

pub fn bounded_log_c(x: f64, Calibration { c, a0, a1, a2 }: Calibration) -> f64 {
    c * bounded_log(x, a0, a1, a2)
}

pub fn bounded_log<S: Scalar + Float>(x: S, a0: S, a1: S, a2: S) -> S {
    let val = a0 + a1 * (x + a2).ln();
    if val > One::one() {
        val
    } else {
        One::one()
    }
}

pub fn bounded_log_v<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S, a2: S) -> DVector<S> {
    x.map(|x| bounded_log(x, a0, a1, a2))
}

pub fn bounded_log_da0<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S, a2: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a0, a1, a2) > one {
            one - one
        } else {
            one
        }
    })
}

pub fn bounded_log_da1<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S, a2: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a0, a1, a2) > one {
            (x + a2).ln()
        } else {
            one
        }
    })
}

pub fn bounded_log_da2<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S, a2: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| {
        if bounded_log(x, a0, a1, a2) > one {
            a1 / (x + a2)
        } else {
            one
        }
    })
}

pub fn fit(data: Vec<(f64, f64)>) -> Calibration {
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    let model = SeparableModelBuilder::<f64>::new(&["a0", "a1", "a2"])
        .function(&["a0", "a1", "a2"], bounded_log_v)
        .partial_deriv("a0", bounded_log_da0)
        .partial_deriv("a1", bounded_log_da1)
        .partial_deriv("a2", bounded_log_da2)
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

    Calibration {
        c: c[0],
        a0: alpha[0],
        a1: alpha[1],
        a2: alpha[2],
    }
}
