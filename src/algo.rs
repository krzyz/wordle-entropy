use crate::structs::{Hint, Hints, KnowledgeN, WordN};
use itertools::izip;

pub fn get_hints<const N: usize>(
    guess: &WordN<N>,
    correct: &WordN<N>,
    knowledge: &KnowledgeN<N>,
) -> (Hints<N>, KnowledgeN<N>) {
    let mut hints: Hints<N> = guess
        .word
        .iter()
        .zip(correct.word.iter())
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

    let known = izip!(&knowledge.known.word, &hints.word, &correct.word)
        .map(|(k, h, c)| {
            if let Some(k) = k {
                Some(*k)
            } else {
                match h {
                    Hint::Right => Some(*c),
                    _ => None,
                }
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    
    let ruled_out = knowledge.ruled_out.clone();
    let out_of_place = knowledge.out_of_place.clone();

    let knowledge = KnowledgeN {
        known,
        out_of_place,
        ruled_out,
    };

    (hints, knowledge)
}
