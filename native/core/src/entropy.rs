use arrayvec::ArrayVec;
use fxhash::FxHashMap;
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cmp::Ordering::Equal;

use crate::{
    algo,
    solvers::expected_turns,
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
    let trans = &dictionary.translator;

    #[cfg(feature = "parallel")]
    let guess_words_iter = guess_words_bytes.par_iter();

    #[cfg(not(feature = "parallel"))]
    let guess_words_iter = guess_words_bytes.iter();

    let entropies = guess_words_iter
        .map(|guess_b| {
            let mut guess_hints = FxHashMap::<_, f64>::default();
            for (correct, probability) in possible_answers
                .iter()
                .map(|&i| (&dictionary.words_bytes[i], &dictionary.probabilities[i]))
            {
                let mut left = ArrayVec::<_, N>::new();
                let hints = algo::get_hints_with_work_array(&guess_b, correct, &mut left);
                *guess_hints.entry(hints).or_default() += *probability / prob_norm;
            }

            let probs = Array1::<f64>::from_vec(guess_hints.values().copied().collect::<Vec<_>>());
            let entropy = entropy(probs);
            let guess = trans.to_chars(guess_b);

            EntropiesData::new(guess.clone(), entropy, guess_hints)
        })
        .collect::<Vec<_>>();

    entropies
}

pub fn entropies_scored<const N: usize>(
    dictionary: &Dictionary<N>,
    answers: &[usize],
    entropies: Vec<EntropiesData<N>>,
) -> Vec<(EntropiesData<N>, f64)> {
    let uncertainty = (dictionary.words.len() as f64).log2();
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
            let left_diff = expected_turns(
                uncertainty - entropies_data.entropy,
                0.,
                1.6369421,
                -0.029045254,
            ) * (1. - prob);

            (entropies_data, left_diff)
        })
        .collect::<Vec<_>>();

    scores.sort_by(|&(_, score1), &(_, score2)| score1.partial_cmp(&score2).unwrap_or(Equal));

    scores
}
