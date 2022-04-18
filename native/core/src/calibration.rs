use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use varpro::model::builder::error::ModelBuildError;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub c: f64,
    pub a0: f64,
    pub a1: f64,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            c: 1.785,
            a0: 0.4273,
            a1: 3.775,
        }
    }
}

pub fn bounded_log_c(x: f64, Calibration { c, a0, a1 }: Calibration) -> f64 {
    let val = c * log(x, a0, a1);

    if val > 1. {
        val
    } else {
        1.
    }
}

pub fn log<S: Scalar + Float>(x: S, a0: S, a1: S) -> S {
    (a0 * (x + a1)).ln()
}

pub fn log_v<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S) -> DVector<S> {
    x.map(|x| log(x, a0, a1))
}

pub fn log_da0<S: Scalar + Float>(x: &DVector<S>, a0: S, a1: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| one / (a0 * (x + a1)))
}

pub fn log_da1<S: Scalar + Float>(x: &DVector<S>, _a0: S, a1: S) -> DVector<S> {
    let one: S = One::one();
    x.map(|x| one / (x + a1))
}

#[derive(Error, Debug)]
pub enum FitError {
    #[error("Unable to build model")]
    ModelBuildUnsuccessful(#[from] ModelBuildError),
    #[error("Unable to build problem")]
    ProblemBuildUnsuccessful,
    #[error("Unable to fit")]
    FitUnsuccessful,
    #[error("no fit result")]
    NoFitResults,
}

pub fn fit(data: Vec<(f64, f64)>, weights: Vec<f64>) -> Result<Calibration, FitError> {
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().unzip();
    let Calibration { a0, a1, .. } = Calibration::default();

    let model = SeparableModelBuilder::<f64>::new(&["a0", "a1"])
        .function(&["a0", "a1"], log_v)
        .partial_deriv("a0", log_da0)
        .partial_deriv("a1", log_da1)
        .build()
        .map_err(FitError::from)?;

    let problem = LevMarProblemBuilder::new()
        .model(&model)
        .x(x)
        .y(y)
        .weights(weights)
        .initial_guess(&[a0, a1])
        .build()
        .map_err(|_| FitError::ProblemBuildUnsuccessful)?;

    let (solved_problem, report) = LevMarSolver::new().minimize(problem);
    if !report.termination.was_successful() {
        return Err(FitError::FitUnsuccessful);
    }

    let alpha = solved_problem.params();
    let c = solved_problem
        .linear_coefficients()
        .ok_or(FitError::NoFitResults)?;

    Ok(Calibration {
        c: c[0],
        a0: alpha[0],
        a1: alpha[1],
    })
}
