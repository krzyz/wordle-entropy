use std::collections::{HashSet};

use crate::structs::{Hint, HintsN, KnowledgeN, PartialChar, WordN};
use fxhash::FxHashMap;
use itertools::izip;
use serde::{Deserialize, Serialize};

pub fn get_hints<T, const N: usize>(guess: &WordN<T, N>, correct: &WordN<T, N>, left: &mut Vec<T>) -> HintsN<N>
where
    T: Serialize + Copy + Eq,
    for<'de2> T: Deserialize<'de2>,
{
    let mut hints = HintsN::<N>::wrong();
    left.clear();
    for (i, (g, c)) in guess.0.into_iter().zip(correct.0.into_iter()).enumerate() {
        if g == c {
            hints.0[i] = Hint::Correct
        } else {
            left.push(c);
        }
    }

    for l in left {
        for i in 0..N {
            if hints.0[i] == Hint::Wrong && *l == guess.0[i] {
                hints.0[i] = Hint::OutOfPlace;
                break;
            }
        }
    }

    hints
}

pub fn update_knowledge<const N: usize>(
    guess: &WordN<char, N>,
    hints: &HintsN<N>,
    knowledge: KnowledgeN<N>,
) -> KnowledgeN<N> {
    let known_now = {
        let mut known_now: FxHashMap<_, u8> = FxHashMap::default();
        for (g, h) in izip!(guess.0, hints.0) {
            match h {
                Hint::Correct | Hint::OutOfPlace => {
                    *known_now.entry(g).or_default() += 1;
                }
                _ => (),
            }
        }
        known_now
    };

    let ruled_out_now = guess
        .0
        .into_iter()
        .zip(hints.0.into_iter())
        .filter(|&(c, h)| h == Hint::Wrong && !known_now.contains_key(&c))
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    let placed = izip!(knowledge.placed.word, &hints.0, &guess.0)
        .map(|(k, &h, &g)| match (k, h) {
            (PartialChar::Some(k), _) => PartialChar::Some(k),
            (_, Hint::Correct) => PartialChar::Some(g),
            (PartialChar::Excluded(mut excluded), _) => {
                excluded.insert(g);
                PartialChar::Excluded(excluded)
            }
            (PartialChar::None, _) => PartialChar::Excluded(HashSet::from([g])),
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let ruled_out = {
        let mut ruled_out = knowledge.ruled_out;
        ruled_out.extend(ruled_out_now);
        ruled_out
    };

    let known = {
        let mut known = knowledge.known;
        for (char, count) in known_now {
            let old_count = known.entry(char).or_default();
            *old_count = count.max(*old_count);
        }
        known
    };

    let knowledge = KnowledgeN {
        known,
        ruled_out,
        placed,
    };

    knowledge
}

pub fn get_hints_and_update<const N: usize>(
    guess: &WordN<char, N>,
    correct: &WordN<char, N>,
    knowledge: KnowledgeN<N>,
) -> (HintsN<N>, KnowledgeN<N>) {
    let mut left = Vec::with_capacity(N);
    let hints = get_hints(guess, correct, &mut left);
    let knowledge = update_knowledge(guess, &hints, knowledge);

    (hints, knowledge)
}

pub fn check<const N: usize>(word: &WordN<char, N>, knowledge: &KnowledgeN<N>) -> bool {
    let mut known_left = knowledge.known.clone();

    for (w, p) in izip!(word.0, &knowledge.placed.word) {
        if knowledge.ruled_out.contains(&w) && *p != PartialChar::Some(w) {
            return false;
        }
        match p {
            PartialChar::Some(c) if *c != w => return false,
            PartialChar::Excluded(excluded) if excluded.contains(&w) => return false,
            _ => (),
        }

        if let Some(count) = known_left.get_mut(&w) {
            *count = count.saturating_sub(1);
        }
    }

    known_left.retain(|_, &mut v| v > 0);

    if known_left.len() > 0 {
        return false;
    }

    return true;
}

pub fn get_answers<const N: usize>(
    words: Vec<WordN<char, N>>,
    knowledge: &KnowledgeN<N>,
) -> Vec<WordN<char, N>> {
    words
        .into_iter()
        .filter(|word| check(word, knowledge))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::Translator;

    use rstest::rstest;
    const WORDS_LENGTH: usize = 5;

    type Word = WordN<char, WORDS_LENGTH>;
    type Hints = HintsN<WORDS_LENGTH>;

    #[rstest]
    #[case("śląsk", "oślik", "OOWWC")]
    #[case("abcde", "abcde", "CCCCC")]
    #[case("abcdd", "abcde", "CCCCW")]
    #[case("aabab", "aaabb", "CCOOC")]
    #[case("aabab", "bxaxx", "OWOWW")]
    #[case("cacbb", "abcba", "WOCCO")]
    fn hints_ok(#[case] guess: &str, #[case] answer: &str, #[case] expected: &str) {
        let guess_w = Word::new(guess);
        let answer_w = Word::new(answer);
        let translator = Translator::generate(&[guess_w.clone(), answer_w.clone()]);
        let guess_b = translator.to_bytes(&guess_w);
        let answer_b = translator.to_bytes(&answer_w);
        let mut left = Vec::with_capacity(WORDS_LENGTH);
        let hints = get_hints(&guess_b, &answer_b, &mut left);
        assert_eq!(Hints::from_str(expected).unwrap(), hints);
    }
}
