use ndarray::Array;
use rand::prelude::IteratorRandom;
use std::{cmp::Ordering::Equal, time::Instant};

use crate::{
    algo::{get_answers, get_hints_and_update},
    entropy::calculate_entropies,
    structs::{HintsN, KnowledgeN, WordN, Dictionary, EntropiesData},
    util::print_vec,
};

pub fn expected_turns(x: f64, r: f64, a: f64, b: f64) -> f64 {
    let x = x + 1.;
    b + a * x.powf(r) * x.ln()
}

fn solve<const N: usize>(
    initial_entropies: &Vec<EntropiesData<N>>,
    dictionary: &Dictionary<N>,
    correct: &WordN<char, N>,
    print: bool,
) -> (Vec<WordN<char, N>>, Vec<HintsN<N>>, Vec<f64>, Vec<f64>) {
    let words = &dictionary.words;
    let mut answers = words.clone();
    let mut knowledge = KnowledgeN::<N>::default();
    let mut total_entropies = Vec::<f64>::new();
    let mut uncertainties= Vec::<f64>::new();
    let mut guesses = vec![];
    let mut all_hints = vec![];
    let mut uncertainty = (words.len() as f64).log2();

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
            calculate_entropies(dictionary, &answers)
        };

        let mut scores = entropies
            .into_iter()
            .map(|entropies_data| {
                let prob = if answers.contains(&entropies_data.word) {
                    1. / (answers.len() as f64)
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(uncertainty - entropies_data.entropy, 0., 1.6369421, -0.029045254) * (1. - prob);

                (entropies_data, left_diff)
            })
            .collect::<Vec<_>>();

        scores.sort_by(|&(_, score1), &(_, score2)| {
            score1.partial_cmp(&score2).unwrap_or(Equal)
        });


        /*
        if print {
            for (word, (entropy, score, _)) in scores.iter().take(10) {
                println!("{word}: {entropy} entropy, {score} score");
            }
        }
        */

        let (entropies_data, _) = scores.into_iter().next().unwrap();
        let guess = &entropies_data.word;
        let probabilities = &entropies_data.probabilities;

        let (hints, knowledge_new) = get_hints_and_update(guess, correct, knowledge);

        let actual_entropy = -probabilities.get(&hints).unwrap().log2();
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

pub fn solve_random<const N: usize>(dictionary: &Dictionary<N>, n: usize) -> Vec<(f64, i32)> {
    let words = &dictionary.words;
    let correct_words = words.iter().choose_multiple(&mut rand::thread_rng(), n);

    let start = Instant::now();
    let initial_entropies = calculate_entropies(dictionary, words);
    let duration = start.elapsed();
    println!("Initial entropies calculation took: {}ms", duration.as_millis());

    let mut turns = vec![];
    let mut unc_data = vec![];

    for correct in correct_words {
        println!("correct: {correct}");
        let (guesses, hints, entropies, uncertainties) = solve(&initial_entropies, dictionary, correct, false);

        print_vec(&guesses);
        print_vec(&hints);
        println!("{entropies:?}");
        println!();

        turns.push(guesses.len() as f64);
        let unc_points = uncertainties.iter().enumerate().map(|(i, unc)| (*unc, (guesses.len() - i) as i32)).collect::<Vec<_>>();
        unc_data.extend(unc_points);
    }

    let turns = Array::from(turns);

    println!("turns: {turns}");
    println!("mean: {}", turns.mean().unwrap());
    unc_data 
}