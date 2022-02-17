use std::io::{self, BufRead};
use std::{fs::File, path::Path};

use crate::structs::WordN;

pub fn load_words<P, const N: usize>(filename: P) -> io::Result<Vec<WordN<N>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    io::BufReader::new(file)
        .lines()
        .map(|l| l.map(|l| WordN::<N>::new(&l)))
        .collect()
}
