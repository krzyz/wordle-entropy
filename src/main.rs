mod algo;
mod data;
mod math;
mod structs;

use algo::{get_answers, get_hints_and_update};
use indexmap::IndexMap;
use math::entropy;
use ndarray::Array1;
use rand::prelude::IteratorRandom;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{cmp::Ordering::Equal, fmt::Display};
use structs::{HintsN, WordN};

use crate::structs::KnowledgeN;

const WORDS_PATH: &str = "/home/krzyz/projects/data/words_polish.txt";
const WORDS_LENGTH: usize = 5;

type Word = WordN<WORDS_LENGTH>;
type Hints = HintsN<WORDS_LENGTH>;
type Knowledge = KnowledgeN<WORDS_LENGTH>;

pub fn print_example() {
    let guess: Word = Word::new("śląsk");
    let correct: Word = Word::new("oślik");
    let knowledge = Knowledge::none();
    let hints = algo::get_hints(&guess, &correct);
    let knowledge = algo::update_knowledge(&guess, &hints, knowledge);

    println!("{hints}");
    println!("{knowledge:#?}");

    let guess: Word = Word::new("rolka");
    let hints = algo::get_hints(&guess, &correct);
    let knowledge = algo::update_knowledge(&guess, &hints, knowledge);
    println!("{hints}");
    println!("{knowledge:#?}");
}

pub fn calculate_entropies<'a, 'b>(
    all_words: &'a Vec<Word>,
    possible_answers: &'b Vec<Word>,
) -> IndexMap<&'a Word, (f32, IndexMap<Hints, f32>)> {
    let n = possible_answers.len() as f32;

    let entropies = {
        let mut entropies = all_words
            .par_iter()
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

        entropies.sort_by(|&_, &(v1, _), &_, &(v2, _)| v2.partial_cmp(&v1).unwrap_or(Equal));
        entropies
    };

    entropies
}

fn solve(
    initial_entropies: &IndexMap<&Word, (f32, IndexMap<Hints, f32>)>,
    words: &Vec<Word>,
    correct: &Word,
    print: bool,
) -> (Vec<Word>, Vec<Hints>, Vec<f32>) {
    let mut answers = words.clone();
    let mut knowledge = Knowledge::default();
    let mut total_entropies = Vec::<f32>::new();
    let mut guesses = vec![];
    let mut all_hints = vec![];

    for i in 0.. {
        if answers.len() == 1 {
            total_entropies.push(total_entropies.last().copied().unwrap_or_default());
            guesses.push(answers.first().unwrap().clone());
            all_hints.push(Hints::correct());
            break;
        }
        let entropies = if i == 0 {
            initial_entropies.clone()
        } else {
            calculate_entropies(&words, &answers)
        };

        if print {
            for (word, (num, _)) in entropies
                .iter()
                .filter(|(g, _)| answers.contains(g))
                .take(10)
            {
                println!("{word}: {num}");
            }
        }

        let (guess, (_, guess_hints)) = entropies
            .into_iter()
            .filter(|(g, _)| answers.contains(g))
            .next()
            .unwrap();
        let (hints, knowledge_new) = get_hints_and_update(&guess, correct, knowledge);
        let actual_entropy = -guess_hints.get(&hints).unwrap().log2();
        knowledge = knowledge_new;
        answers = get_answers(words.clone(), &knowledge);

        total_entropies.push(total_entropies.last().copied().unwrap_or_default() + actual_entropy);
        guesses.push(guess.clone());
        all_hints.push(hints.clone());

        if print {
            println!("next_guess : {guess}");
            println!("hint: {hints}");
            println!("Actual entropy: {actual_entropy}");
            println!("possibilities: {}", answers.len());
            println!(
                "total entropy: {}",
                total_entropies.last().copied().unwrap_or_default()
            );
        }
        if hints == Hints::correct() {
            break;
        }
    }

    (guesses, all_hints, total_entropies)
}

pub fn print_vec<T: Display>(words: &Vec<T>) {
    for word in words {
        println!("{word}")
    }
}

fn main() {
    let words = data::load_words::<_, WORDS_LENGTH>(WORDS_PATH).unwrap();
    let correct_words = words.iter().choose_multiple(&mut rand::thread_rng(), 10);
    println!("correct:");
    print_vec(&correct_words);

    let initial_entropies = calculate_entropies(&words, &words);

    for correct in correct_words {
        let (guesses, hints, entropies) = solve(&initial_entropies, &words, correct, false);

        print_vec(&guesses);
        print_vec(&hints);
        println!("{entropies:?}");
        println!();
    }
}
