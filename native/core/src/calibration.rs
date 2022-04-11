use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use thiserror::Error;
use varpro::model::builder::error::ModelBuildError;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

#[derive(Copy, Clone, Debug)]
pub struct Calibration {
    pub c: f64,
    pub a0: f64,
    pub a1: f64,
    pub a2: f64,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            c: 0.662,
            a0: -1.796,
            a1: 2.496,
            a2: 2.942,
        }
    }
}

pub fn bounded_log_c(x: f64, Calibration { c, a0, a1, a2 }: Calibration) -> f64 {
    let val = c * bounded_log(x, a0, a1, a2);
    if val > 1. {
        val
    } else {
        1.
    }
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

#[derive(Error, Debug)]
pub enum FitError {
    #[error("Unable to build model")]
    ModelBuildUnsuccessful(#[from] ModelBuildError),
    #[error("Unable to build problem")]
    ProblemBuildUnsuccessful,
    #[error("Unable to fit")]
    FitUnsuccessful,
}

pub fn fit(data: Vec<(f64, f64)>, weights: Vec<f64>) -> Result<Calibration, FitError> {
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().unzip();
    let Calibration { a0, a1, a2, .. } = Calibration::default();

    let model = SeparableModelBuilder::<f64>::new(&["a0", "a1", "a2"])
        .function(&["a0", "a1", "a2"], bounded_log_v)
        .partial_deriv("a0", bounded_log_da0)
        .partial_deriv("a1", bounded_log_da1)
        .partial_deriv("a2", bounded_log_da2)
        .build()
        .map_err(FitError::from)?;

    let problem = LevMarProblemBuilder::new()
        .model(&model)
        .x(x)
        .y(y)
        .weights(weights)
        .initial_guess(&[a0, a1, a2])
        .build()
        .map_err(|_| FitError::ProblemBuildUnsuccessful)?;

    let (solved_problem, report) = LevMarSolver::new().minimize(problem);
    if !report.termination.was_successful() {
        return Err(FitError::FitUnsuccessful);
    }

    let alpha = solved_problem.params();
    let c = solved_problem.linear_coefficients().unwrap();

    Ok(Calibration {
        c: c[0],
        a0: alpha[0],
        a1: alpha[1],
        a2: alpha[2],
    })
}
