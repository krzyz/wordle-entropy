use arrayvec::ArrayVec;
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cmp::Ordering::Equal;

use crate::{
    algo,
    calibration::{bounded_log_c, Calibration},
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
) -> Vec<EntropiesData<N>> {
    let prob_norm: f64 = possible_answers
        .iter()
        .map(|&i| dictionary.probabilities[i])
        .sum();
    let guess_words_bytes = &dictionary.words_bytes;

    #[cfg(feature = "parallel")]
    let guess_words_iter = guess_words_bytes.par_iter();

    #[cfg(not(feature = "parallel"))]
    let guess_words_iter = guess_words_bytes.iter();

    let entropies = guess_words_iter
        .map(|guess_b| {
            let mut guess_hints = vec![0.; dictionary.hints.len()];
            for (correct, probability) in possible_answers
                .iter()
                .map(|&i| (&dictionary.words_bytes[i], &dictionary.probabilities[i]))
            {
                let mut left = ArrayVec::<_, N>::new();
                let hints = algo::get_hints_with_work_array(&guess_b, correct, &mut left);
                guess_hints[hints.to_ind()] += *probability / prob_norm;
            }

            let probs = Array1::<f64>::from_vec(guess_hints.clone());
            let entropy = entropy(probs);

            EntropiesData::new(entropy, guess_hints)
        })
        .collect::<Vec<_>>();

    entropies
}

pub fn entropies_scored<const N: usize>(
    dictionary: &Dictionary<N>,
    answers: &[usize],
    entropies: Vec<EntropiesData<N>>,
    uncertainty: Option<f64>,
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
            let left_diff = prob
                + (1.
                    + bounded_log_c(uncertainty - entropies_data.entropy, Calibration::default()))
                    * (1. - prob);

            (i, entropies_data, left_diff)
        })
        .collect::<Vec<_>>();

    scores.sort_by(|&(_, _, score1), &(_, _, score2)| score1.partial_cmp(&score2).unwrap_or(Equal));

    scores
}
