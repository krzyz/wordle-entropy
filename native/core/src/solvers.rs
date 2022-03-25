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
    let x = f64::max(x, 0.) + 1.;
    b + a * x.powf(r) * x.ln() + 1.
}

pub fn solve<const N: usize>(
    initial_entropies: &Vec<EntropiesData<N>>,
    dictionary: &Dictionary<N>,
    correct: &WordN<char, N>,
    print: bool,
) -> (Vec<WordN<char, N>>, Vec<HintsN<N>>, Vec<f64>, Vec<f64>) {
    let words = &dictionary.words;
    let mut answers = (0..words.len()).collect::<Vec<_>>();
    let mut knowledge = KnowledgeN::<N>::default();
    let mut total_information= Vec::<f64>::new();
    let mut uncertainties= Vec::<f64>::new();
    let mut guesses = vec![];
    let mut all_hints = vec![];
    let full_information = (words.len() as f64).log2();
    let mut uncertainty = full_information; 
    let mut prob_norm: f64 =  answers.iter().map(|&i| dictionary.probabilities[i]).sum();

    for i in 0.. {
        uncertainties.push(uncertainty);
        if answers.len() == 1 {
            total_information.push(total_information.last().copied().unwrap_or_default());
            guesses.push(dictionary.words[*answers.first().unwrap()].clone());
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
            .enumerate()
            .map(|(i, entropies_data)| {
                let prob = if answers.contains(&i) {
                    dictionary.probabilities[i] / prob_norm
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

        if print {
            println!("Prob norm: {prob_norm}");
            println!("10 best gueses:");
            for (entropies_data, score) in scores.iter().take(10) {
                println!("{}: {score}", entropies_data.word);
            }
        }

        let (entropies_data, _) = scores.into_iter().next().unwrap();
        let guess = &entropies_data.word;

        let (hints, knowledge_new) = get_hints_and_update(guess, correct, knowledge);

        knowledge = knowledge_new;
        answers = get_answers(words.clone(), &knowledge);
        
        prob_norm = answers.iter().map(|&i| dictionary.probabilities[i]).sum();

        uncertainty = answers.iter().map(|&i| {
            let probability = dictionary.probabilities[i]/prob_norm;
            -probability * probability.log2()
        }).sum();

        total_information.push(full_information - uncertainty);

        let last_total_information = total_information.last().copied().unwrap_or_default();

        guesses.push(guess.clone());
        all_hints.push(hints.clone());

        if print {
            println!("next_guess : {guess}, hints: {hints}");
            println!("possibilities: {}", answers.len());
            if answers.len() < 10 {
                for &i in &answers {
                    println!("{}: {}", dictionary.words[i], dictionary.probabilities[i]/prob_norm);
                }
            }
            println!(
                "uncertainty: {uncertainty}, total information: {last_total_information}"
            );
            println!("knowledge: {knowledge:?}");
        }
        if hints == HintsN::<N>::correct() {
            break;
        }
    }

    (guesses, all_hints, total_information, uncertainties)
}

pub fn solve_random<const N: usize>(dictionary: &Dictionary<N>, n: usize) -> Vec<(f64, i32)> {
    let words = &dictionary.words;
    let answers = (0..words.len()).collect::<Vec<_>>();
    let correct_words = words.iter().choose_multiple(&mut rand::thread_rng(), n);

    let start = Instant::now();
    let initial_entropies = calculate_entropies(dictionary, &answers);
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