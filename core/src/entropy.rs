use fxhash::FxHashMap;
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    algo,
    structs::{WordN, Entropies}, translator::Translator,
};

pub fn entropy(arr: Array1<f32>) -> f32 {
    let arr = arr
        .into_iter()
        .filter(|&x| x > 0.)
        .map(|x| x as f32)
        .collect::<Array1<f32>>();

    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.mapv_inplace(|x| (x).log2());
        arr_log
    };


    -1. * (arr * arr_log).sum()
}

pub fn calculate_entropies<'a, 'b, const N: usize>(
    all_words: &'a Vec<WordN<char, N>>,
    possible_answers: &'b Vec<WordN<char, N>>,
) -> Entropies<N> {
    let n = possible_answers.len() as f32;

    let trans_all = Translator::generate(&all_words[..]);
    let trans_ans = Translator::generate(&possible_answers[..]);

    let possible_answers: Vec<_> = possible_answers.iter().map(|w| trans_ans.to_bytes(w)).collect();

    #[cfg(feature = "parallel")]
    let all_words_iter = all_words.par_iter().with_min_len(1000);

    #[cfg(not(feature = "parallel"))]
    let all_words_iter = all_words.iter();

    let entropies = all_words_iter
        .map(|guess| {
            let guess_b = trans_all.to_bytes(guess);
            let mut guess_hints = FxHashMap::<_, f32>::default();
            for correct in possible_answers.iter() {
                let hints = algo::get_hints(&guess_b, correct);
                *guess_hints.entry(hints).or_default() += 1. / n;
            }

            let probs = Array1::<f32>::from_vec(
                guess_hints.values().map(|x| *x as f32).collect::<Vec<_>>(),
            );
            let entropy = entropy(probs);

            (*guess, (entropy, guess_hints))
        })
        .collect::<Vec<_>>();

    entropies
}
