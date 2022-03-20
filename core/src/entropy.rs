use fxhash::{FxHashMap, FxHashSet};
use ndarray::Array1;
#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    algo,
    structs::{Entropies, WordN},
    translator::Translator,
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
) -> Entropies<N> {
    let n = possible_answers.len() as f32;

    let all_words = guess_words
        .into_iter()
        .chain(possible_answers.into_iter())
        .map(|&x| x)
        .collect::<FxHashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let trans = Translator::generate(&all_words[..]);

    let possible_answers: Vec<_> = possible_answers.iter().map(|w| trans.to_bytes(w)).collect();

    #[cfg(feature = "parallel")]
    let guess_words_iter = guess_words.par_iter().with_min_len(1000);

    #[cfg(not(feature = "parallel"))]
    let guess_words_iter = guess_words.iter();

    let entropies = guess_words_iter
        .map(|guess| {
            let guess_b = trans.to_bytes(guess);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use rstest::rstest;
    const WORDS_LENGTH: usize = 5;

    type Word = WordN<char, WORDS_LENGTH>;

    const WORDS_PATH: &str = "/home/krzyz/projects/data/words_polish.txt";

    #[rstest]
    #[case("korea", 5.8375506)]
    fn entropy_ok(#[case] guess: &str, #[case] expected: f32) {
        let guess = Word::new(guess);
        let words = data::load_words::<_, WORDS_LENGTH>(WORDS_PATH).unwrap();

        let entropies = calculate_entropies(&vec![guess], &words);

        assert_eq!(1, entropies.len());
        assert_eq!(expected, entropies.get(0).unwrap().1 .0);

        //let (_, (entropy, _)) = calculate_entropies(&words, &words).into_iter().find(|&(w, _)| w == Word::new("korea")).unwrap();

        //assert_eq!(expected, entropy);
    }
}
