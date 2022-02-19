use indexmap::IndexMap;
use ndarray::Array;
use rand::prelude::IteratorRandom;
use std::cmp::Ordering::Equal;

use crate::{
    algo::{get_answers, get_hints_and_update},
    entropy::calculate_entropies,
    structs::{HintsN, KnowledgeN, WordN},
    util::print_vec,
};

pub fn expected_turns(entropy_left: f32) -> f32 {
    2. / 9. * entropy_left + 1.
}

fn solve<const N: usize>(
    initial_entropies: &IndexMap<&WordN<N>, (f32, IndexMap<HintsN<N>, f32>)>,
    words: &Vec<WordN<N>>,
    correct: &WordN<N>,
    print: bool,
) -> (Vec<WordN<N>>, Vec<HintsN<N>>, Vec<f32>, Vec<f32>) {
    let mut answers = words.clone();
    let mut knowledge = KnowledgeN::<N>::default();
    let mut total_entropies = Vec::<f32>::new();
    let mut uncertainties= Vec::<f32>::new();
    let mut guesses = vec![];
    let mut all_hints = vec![];
    let mut uncertainty = (words.len() as f32).log2();

    for i in 0.. {
        uncertainties.push(uncertainty);
        if answers.len() == 1 {
            total_entropies.push(total_entropies.last().copied().unwrap_or_default());
            guesses.push(answers.first().unwrap().clone());
            all_hints.push(HintsN::<N>::correct());
            break;
        }
        let entropies = if i == 0 {
            initial_entropies.clone()
        } else {
            calculate_entropies(&words, &answers)
        };

        let mut scores = entropies
            .into_iter()
            .map(|(g, (entropy, guess_hints))| {
                let prob = if answers.contains(g) {
                    1. / (answers.len() as f32)
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(uncertainty - entropy) * (1. - prob);

                (g, (entropy, left_diff, guess_hints))
            })
            .collect::<IndexMap<_, _>>();

        scores.sort_by(|&_, &(_, score1, _), &_, &(_, score2, _)| {
            score1.partial_cmp(&score2).unwrap_or(Equal)
        });

        if print {
            for (word, (entropy, score, _)) in scores.iter().take(10) {
                println!("{word}: {entropy} entropy, {score} score");
            }
        }

        let (guess, (_, _, guess_hints)) = scores.into_iter().next().unwrap();

        let (hints, knowledge_new) = get_hints_and_update(&guess, correct, knowledge);

        let actual_entropy = -guess_hints.get(&hints).unwrap().log2();
        knowledge = knowledge_new;
        answers = get_answers(words.clone(), &knowledge);

        total_entropies.push(total_entropies.last().copied().unwrap_or_default() + actual_entropy);
        uncertainty -= actual_entropy;
        guesses.push(guess.clone());
        all_hints.push(hints.clone());

        if print {
            println!("next_guess : {guess}, hints: {hints}");
            println!("Actual entropy: {actual_entropy}");
            println!("possibilities: {}", answers.len());
            println!(
                "uncertainty: {uncertainty}, total entropy: {}",
                total_entropies.last().copied().unwrap_or_default()
            );
            println!("knowledge: {knowledge:?}");
        }
        if hints == HintsN::<N>::correct() {
            break;
        }
    }

    (guesses, all_hints, total_entropies, uncertainties)
}

pub fn solve_random<const N: usize>(words: &Vec<WordN<N>>, n: usize) {
    let correct_words = words.iter().choose_multiple(&mut rand::thread_rng(), n);

    let initial_entropies = calculate_entropies(words, words);

    let mut turns = vec![];

    for correct in correct_words {
        println!("correct: {correct}");
        let (guesses, hints, entropies, uncertainties) = solve(&initial_entropies, words, correct, false);

        print_vec(&guesses);
        print_vec(&hints);
        println!("{entropies:?}");
        println!();

        turns.push(guesses.len() as f32);
    }

    let turns = Array::from(turns);

    println!("turns: {turns}");
    println!("mean: {}", turns.mean().unwrap());
}
