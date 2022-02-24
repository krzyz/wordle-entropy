use indexmap::IndexMap;
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    algo,
    structs::{HintsN, WordN},
};

/*

pub fn norm_entropy(arr: Array1<i32>, n: i32) -> f32 {
    let n = n as f32;
    let arr = arr
        .into_iter()
        .filter(|&x| x > 1)
        .map(|x| x as f32)
        .collect::<Array1<f32>>();

    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.par_mapv_inplace(|x| x.log(n));
        arr_log
    };

    1. - 1. / (n) * (arr * arr_log).sum()
}
*/

pub fn entropy(arr: Array1<f32>) -> f32 {
    let arr = arr
        .into_iter()
        .filter(|&x| x > 0.)
        .map(|x| x as f32)
        .collect::<Array1<f32>>();

    #[cfg(feature = "parallel")]
    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.par_mapv_inplace(|x| (x).log2());
        arr_log
    };

    #[cfg(not(feature = "parallel"))]
    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.mapv_inplace(|x| (x).log2());
        arr_log
    };


    -1. * (arr * arr_log).sum()
}

pub fn calculate_entropies<'a, 'b, const N: usize>(
    all_words: &'a Vec<WordN<N>>,
    possible_answers: &'b Vec<WordN<N>>,
) -> IndexMap<&'a WordN<N>, (f32, IndexMap<HintsN<N>, f32>)> {
    let n = possible_answers.len() as f32;

    #[cfg(feature = "parallel")]
    let all_words_iter = all_words.par_iter();

    #[cfg(not(feature = "parallel"))]
    let all_words_iter = all_words.iter();

    let entropies = all_words_iter
        .map(|guess| {
            let mut guess_hints = IndexMap::<_, f32>::new();
            for correct in possible_answers.iter() {
                let hints = algo::get_hints(guess, correct);
                *guess_hints.entry(hints).or_default() += 1. / n;
            }

            let probs = Array1::<f32>::from_vec(
                guess_hints.values().map(|x| *x as f32).collect::<Vec<_>>(),
            );
            let entropy = entropy(probs);

            (guess, (entropy, guess_hints))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<IndexMap<_, (_, _)>>();

    entropies
}
