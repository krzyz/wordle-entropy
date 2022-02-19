mod algo;
mod data;
mod entropy;
mod solvers;
mod structs;
mod util;

use solvers::solve_random;
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

fn main() {
    let words = data::load_words::<_, WORDS_LENGTH>(WORDS_PATH).unwrap();

    solve_random(&words, 500);
}
