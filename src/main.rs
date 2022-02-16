mod algo;
mod data;
mod math;
mod structs;

use indexmap::IndexMap;
use math::entropy;
use ndarray::Array1;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cmp::Ordering::Equal;
use structs::WordN;

use crate::structs::KnowledgeN;

const WORDS_PATH: &str = "/home/krzyz/projects/data/words_polish.txt";
const WORDS_LENGTH: usize = 5;

type Word = WordN<WORDS_LENGTH>;
type Knowledge = KnowledgeN<WORDS_LENGTH>;

pub fn print_example() {
    let guess: Word = Word::new("śląsk");
    let correct: Word = Word::new("oślik");
    let knowledge = Knowledge::none();
    let (hints, knowledge) = algo::get_hints(&guess, &correct, &knowledge);

    println!("{hints}");
    let known = &knowledge.known;
    println!("{known:#?}");

    let guess: Word = Word::new("rolka");
    let (hints, knowledge) = algo::get_hints(&guess, &correct, &knowledge);
    println!("{hints}");
    let known = &knowledge.known;
    println!("{known:#?}");
}

fn main() {
    let words = data::load_words::<_, WORDS_LENGTH>(WORDS_PATH).unwrap();
    let n = words.len() as i32;

    let knowledge = Knowledge::none();

    let entropies = {
        let mut entropies = words.par_iter().map(|guess| {
            let mut guess_hints = IndexMap::<_, i32>::new();
            for correct in words.iter() {
                let (hints, _) = algo::get_hints(guess, correct, &knowledge);
                *guess_hints.entry(hints).or_default() += 1;
            }

            let arr = Array1::<i32>::from_vec(guess_hints.values().map(|x| *x).collect::<Vec<_>>());
            let entropy = entropy(arr, n);

            (guess, entropy)
        }).collect::<Vec<_>>().into_iter().collect::<IndexMap<_, f32>>();

        entropies.sort_by(|&_, &v1, &_, &v2| v2.partial_cmp(&v1).unwrap_or(Equal));
        entropies
    };

    for (word, num) in entropies.iter().take(10) {
        println!("{word}: {num}");
    }
}
