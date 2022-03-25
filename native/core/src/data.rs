use crate::structs::{Dictionary, WordN};
use std::io::{self, BufRead};
use std::{fs::File, path::Path};

pub fn load_words<P, const N: usize>(filename: P) -> io::Result<Dictionary<N>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let words = io::BufReader::new(file)
        .lines()
        .map(|l| l.map(|l| WordN::<char, N>::new(&l)))
        .collect::<io::Result<Vec<_>>>()?;

    Ok(Dictionary::new(words))
}

pub fn parse_words<'a, I, const N: usize>(lines: I) -> Vec<WordN<char, N>>
where
    I: Iterator<Item = &'a str>,
{
    lines.map(|l| WordN::<char, N>::new(&l)).collect()
}
