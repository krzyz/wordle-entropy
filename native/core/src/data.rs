use crate::structs::{Dictionary, WordN};
use std::io::{self, BufRead};
use std::{fs::File, path::Path};

pub fn load_words<P, const N: usize>(filename: P) -> io::Result<Dictionary<N>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let words_with_probs = io::BufReader::new(file)
        .lines()
        .map(|l| {
            l.map(|l| {
                let mut split = l.split(",");
                let (word_str, probability_str) = (split.next().unwrap(), split.next().unwrap());
                (
                    WordN::<char, N>::new(&word_str),
                    probability_str.parse::<f64>().unwrap(),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    let (words, probabilities) = words_with_probs.into_iter().unzip();

    Ok(Dictionary::new(words, probabilities))
}

pub fn parse_words<'a, I, const N: usize>(lines: I) -> Dictionary<N>
where
    I: Iterator<Item = &'a str>,
{
    let words_with_probs = lines
        .map(|l| {
            let mut split = l.split(",");
            let (word_str, probability_str) = (split.next().unwrap(), split.next().unwrap());
            (
                WordN::<char, N>::new(&word_str),
                probability_str.parse::<f64>().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let (words, probabilities) = words_with_probs.into_iter().unzip();

    Dictionary::new(words, probabilities)
}
