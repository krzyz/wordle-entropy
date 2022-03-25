use arrayvec::ArrayVec;
use fxhash::{FxHashMap, FxHashSet};
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    algo,
    structs::{HintsN, WordN}, translator::Translator,
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
    guess_words: &'a Vec<WordN<char, N>>,
    possible_answers: &'b Vec<WordN<char, N>>,
) -> Vec<(WordN<char, N>, (f32, FxHashMap<HintsN<N>, f32>))> {
    let n = possible_answers.len() as f32;

    let all_words = guess_words
        .into_iter()
        .chain(possible_answers.into_iter())
        .map(|x| x.clone())
        .collect::<FxHashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let trans = Translator::generate(&all_words[..]);

    let possible_answers: Vec<_> = possible_answers.iter().map(|w| trans.to_bytes(w)).collect();

    #[cfg(feature = "parallel")]
    let guess_words_iter = guess_words.par_iter();

    #[cfg(not(feature = "parallel"))]
    let guess_words_iter = guess_words.iter();

    let entropies = guess_words_iter
        .map(|guess| {
            let guess_b = trans.to_bytes(guess);
            let mut guess_hints = FxHashMap::<_, f32>::default();
            for correct in possible_answers.iter() {
                let mut left = ArrayVec::<_, N>::new();
                let hints = algo::get_hints_with_work_array(&guess_b, correct, &mut left);
                *guess_hints.entry(hints).or_default() += 1. / n;
            }

            let probs = Array1::<f32>::from_vec(
                guess_hints.values().map(|x| *x as f32).collect::<Vec<_>>(),
            );
            let entropy = entropy(probs);

            (guess.clone(), (entropy, guess_hints))
        })
        .collect::<Vec<_>>();

    entropies
}
