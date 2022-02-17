use std::collections::{HashMap, HashSet};

use crate::structs::{Hint, HintsN, KnowledgeN, PartialChar, WordN};
use itertools::izip;

pub fn get_hints<const N: usize>(
    guess: &WordN<N>,
    correct: &WordN<N>) -> HintsN<N> {
    let mut hints: HintsN<N> = guess
        .word
        .into_iter()
        .zip(correct.word.into_iter())
        .map(|(g, c)| if g == c { Hint::Right } else { Hint::Wrong })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let mut left = correct
        .word
        .iter()
        .zip(hints.word.into_iter())
        .filter(|&(_, h)| h == Hint::Wrong)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    for (i, g) in guess.word.iter().enumerate() {
        if let Some(index) = left.iter().position(|&l| g == l) {
            hints.word[i] = Hint::OutOfPlace;
            left.remove(index);
        }
    }
    hints
}

pub fn update_knowledge<const N: usize>(
    guess: &WordN<N>,
    hints: &HintsN<N>,
    knowledge: KnowledgeN<N>,
) -> KnowledgeN<N> {
    let ruled_out_now = guess
        .word
        .into_iter()
        .zip(hints.word.into_iter())
        .filter(|&(_, h)| h == Hint::Wrong)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    let known_now = {
        let mut known_now: HashMap<_, u8> = HashMap::new();
        for (g, h) in izip!(guess.word, hints.word) {
            match h {
                Hint::Right | Hint::OutOfPlace => {
                    *known_now.entry(g).or_default() += 1;
                }
                _ => (),
            }
        }
        known_now
    };

    let placed = izip!(knowledge.placed.word, &hints.word, &guess.word)
        .map(|(k, &h, &g)| match (k, h) {
            (PartialChar::Some(k), _) => PartialChar::Some(k),
            (_, Hint::Right) => PartialChar::Some(g),
            (PartialChar::Excluded(mut excluded), Hint::OutOfPlace) => {
                excluded.insert(g);
                PartialChar::Excluded(excluded)
            }
            (PartialChar::Excluded(excluded), Hint::Wrong) => PartialChar::Excluded(excluded),
            (PartialChar::None, Hint::OutOfPlace) => PartialChar::Excluded(HashSet::from([g])),
            (_, _) => PartialChar::None,
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
    guess: &WordN<N>,
    correct: &WordN<N>,
    knowledge: KnowledgeN<N>,
) -> (HintsN<N>, KnowledgeN<N>) {
    let hints = get_hints(guess, correct);
    let knowledge = update_knowledge(guess, &hints, knowledge);

    (hints, knowledge)
}

pub fn check<const N: usize>(word: &WordN<N>, knowledge: &KnowledgeN<N>) -> bool {
    let mut known_left = knowledge.known.clone();

    for (w, p) in izip!(word.word, &knowledge.placed.word) {
        if knowledge.ruled_out.contains(&w) {
            return false
        }
        match p {
            PartialChar::Some(c) if *c != w => {
                return false
            }
            PartialChar::Excluded(excluded) if excluded.contains(&w) => {
                return false
            }
            _ => (),
        }

        if let Some(count) = known_left.get_mut(&w) {
            *count -= 1;
        }
    }

    known_left.retain(|_, &mut v| v > 0);

    if known_left.len() > 0 {
        return false;
    }

    return true;
}

pub fn get_answers<const N: usize>(words: Vec<WordN<N>>, knowledge: &KnowledgeN<N>) -> Vec<WordN<N>> {
    words.into_iter().filter(|word| check(word, knowledge)).collect()
}
