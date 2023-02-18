use std::env;

use nalgebra::{DVector, Scalar};
use num::One;
use num_traits::Float;
use plotters::prelude::*;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};
use we_core::algo;
use we_core::data;
use we_core::solvers::solve_random;
use we_core::structs::{knowledge::KnowledgeN, word::WordN, EntropiesCacheN};
use wordle_entropy_core as we_core;

const WORDS_LENGTH: usize = 5;

type Word = WordN<char, WORDS_LENGTH>;
type Knowledge = KnowledgeN<WORDS_LENGTH>;
type EntropiesCache = EntropiesCacheN<WORDS_LENGTH>;

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

pub fn print_example() {
    let guess: Word = Word::try_from("śląsk").unwrap();
    let correct: Word = Word::try_from("oślik").unwrap();
    let knowledge = Knowledge::none();
    let hints = algo::get_hints(&guess, &correct);
    let knowledge = algo::update_knowledge(&guess, &hints, knowledge);

    println!("{hints}");
    println!("{knowledge:#?}");

    let guess: Word = Word::try_from("rolka").unwrap();
    let hints = algo::get_hints(&guess, &correct);
    let knowledge = algo::update_knowledge(&guess, &hints, knowledge);
    println!("{hints}");
    println!("{knowledge:#?}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("Missing argument: path to word list");
        std::process::exit(1);
    }

    let words_path = &args[1];

    let dictionary = data::load_words::<_, WORDS_LENGTH>(words_path).unwrap();
    let words = &dictionary.words;

    let mut entropies_cache = EntropiesCache::new();

    let unc_data = solve_random(&dictionary, 200, &mut entropies_cache)
        .into_iter()
        .map(|(x, y)| (num::clamp(x, 0., f64::MAX), y as f64))
        .collect::<Vec<_>>();
    // println!("uncertaintes points: {unc_data:?}");

    let (a, b, r) = fit(unc_data.clone());
    println!("{r}, {a}, {b}");

    let y_max = 1.
        + unc_data
            .iter()
            .map(|&(_, left)| left)
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap_or(7.) as f64;
    let x_max = (words.len() as f64).log2() + 1.;

    let root = BitMapBackend::new("/tmp/0.png", (1000, 700)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.margin(10, 10, 10, 10);
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5u32)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(0f64..x_max, 0f64..y_max)?;

    chart.configure_mesh().draw()?;

    let c = 5.;
    chart.draw_series(LineSeries::new(
        (0..=((c * x_max.floor()) as i32))
            .map(|x| (x as f64) / c)
            .map(|x| (x, log_f_s(x, r, a, b))),
        &RED,
    ))?;
    //chart.draw_series(LineSeries::new((0..=(x_max.floor() as i32)).map(|x| (x as f64, (x*x) as f64)), &RED))?;

    chart.draw_series(PointSeries::of_element(
        unc_data,
        2,
        &BLACK,
        &|c, s: i32, st| {
            return Circle::new(c, s, st.filled());
        },
    ))?;

    println!("ok");

    Ok(())
}
