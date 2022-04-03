use crate::structs::WordError;
use crate::structs::{Dictionary, WordN};
use std::io::{self, BufRead};
use std::num::ParseFloatError;
use std::{fs::File, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Error reading file")]
    IOError(#[from] io::Error),
    #[error("Empty line")]
    EmptyLine,
    #[error("Error parsing word")]
    ParseWordError(#[from] WordError),
    #[error("Error parsing probability")]
    ParseFloatError(#[from] ParseFloatError),
}

pub fn load_words<P, const N: usize>(filename: P) -> Result<Dictionary<N>, LoadError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let lines: Vec<_> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    parse_words(lines.iter().map(|s| s.as_ref()))
}

pub fn parse_words<'a, I, const N: usize>(lines: I) -> Result<Dictionary<N>, LoadError>
where
    I: Iterator<Item = &'a str>,
{
    let words_with_probs = lines
        .map(|l| {
            let mut split = l.split(",");
            split
                .next()
                .ok_or_else(|| LoadError::EmptyLine)
                .and_then(|word_str| {
                    let probability_str = split.next().unwrap_or("1.0");
                    word_str
                        .try_into()
                        .map_err(|e: WordError| e.into())
                        .and_then(|word: WordN<char, N>| {
                            probability_str
                                .parse::<f64>()
                                .map_err(|e| e.into())
                                .map(|probability| (word, probability))
                        })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (words, probabilities) = words_with_probs.into_iter().unzip();

    Ok(Dictionary::new(words, probabilities))
}
