use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::{cmp::Ordering::Equal, time::Instant};

use crate::{
    calibration::{bounded_log_c, Calibration},
    hints_computed::HintsComputed,
    structs::{Dictionary, EntropiesData},
};

pub fn entropy(arr: Array1<f64>) -> f64 {
    let arr = arr
        .into_iter()
        .filter(|&x| x > 0.)
        .map(|x| x as f64)
        .collect::<Array1<f64>>();

    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.mapv_inplace(|x| (x).log2());
        arr_log
    };

    -1. * (arr * arr_log).sum()
}

pub fn calculate_entropies<const N: usize>(
    dictionary: &Dictionary<N>,
    possible_answers: &[usize],
    hints_computed: &HintsComputed,
) -> Vec<EntropiesData<N>> {
    let start = Instant::now();
    let prob_norm: f64 = possible_answers
        .iter()
        .map(|&i| dictionary.probabilities[i])
        .sum();

    #[cfg(feature = "parallel")]
    let guess_words_iter = {
        // parallelization has much more overhead running in web worker
        #[cfg(target_arch = "wasm32")]
        let min_len = if possible_answers.len() > 1000 {
            0
        } else {
            dictionary.words.len()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let min_len = 0;

        (0..dictionary.words.len())
            .into_par_iter()
            .with_min_len(min_len)
    };

    #[cfg(not(feature = "parallel"))]
    let guess_words_iter = (0..dictionary.words);

    let res = guess_words_iter
        .map(&|i| {
            assert!(i < hints_computed.size());
            let mut guess_hints = vec![0.; dictionary.hints.len()];
            for &j in possible_answers.into_iter() {
                assert!(j < hints_computed.size());
                let probability = &dictionary.probabilities[j];
                let hints = unsafe { hints_computed.get_hint(i, j) };
                guess_hints[hints] += *probability / prob_norm;
            }

            let probs = Array1::<f64>::from_vec(guess_hints.clone());
            let entropy = entropy(probs);

            EntropiesData::new(entropy, guess_hints)
        })
        .collect::<Vec<_>>();

    let duration = start.elapsed();
    println!(
        "Calculate entropies took: {}ms for {} possible answers and {} words",
        duration.as_millis(),
        possible_answers.len(),
        dictionary.words.len()
    );

    res
}

pub fn entropies_scored<const N: usize>(
    dictionary: &Dictionary<N>,
    answers: &[usize],
    entropies: Vec<EntropiesData<N>>,
    uncertainty: Option<f64>,
    calibration: Option<Calibration>,
) -> Vec<(usize, EntropiesData<N>, f64)> {
    let uncertainty = match uncertainty {
        Some(uncertainty) => uncertainty,
        None => (dictionary.words.len() as f64).log2(),
    };
    let prob_norm: f64 = answers.iter().map(|&i| dictionary.probabilities[i]).sum();

    let mut scores = entropies
        .into_iter()
        .enumerate()
        .map(|(i, entropies_data)| {
            let prob = if answers.contains(&i) {
                dictionary.probabilities[i] / prob_norm
            } else {
                0.
            };

            // the less the better
            // we add 1 to bounded_log_c because we assume the guess is not correct
            // so we must take at least one more turn
            let calibration = calibration.unwrap_or_default();
            let left_diff = prob
                + (1. + bounded_log_c(uncertainty - entropies_data.entropy, calibration))
                    * (1. - prob);

            (i, entropies_data, left_diff)
        })
        .collect::<Vec<_>>();

    scores.sort_by(|&(_, _, score1), &(_, _, score2)| score1.partial_cmp(&score2).unwrap_or(Equal));

    scores
}
