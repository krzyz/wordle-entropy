use std::io::{self, BufRead};
use std::{fs::File, path::Path};

use crate::structs::WordN;

pub fn load_words<P, const N: usize>(filename: P) -> io::Result<Vec<WordN<char, N>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    io::BufReader::new(file)
        .lines()
        .map(|l| l.map(|l| WordN::<char, N>::new(&l)))
        .collect()
}

pub fn parse_words<'a, I, const N: usize>(lines: I) -> Vec<WordN<char, N>>
where I: Iterator<Item = &'a str> {
    lines
        .map(|l| WordN::<char, N>::new(&l))
        .collect()
}