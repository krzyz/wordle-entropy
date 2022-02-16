use core::fmt;
use std::iter;
use colored::Colorize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hint {
    Wrong,
    OutOfPlace,
    Right,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Hints<const N: usize> {
    pub word: [Hint; N],
}
/*
impl<const N: usize> Hints<N> {
    pub fn none() -> Hints<N> {
        Hints {
            word: iter::repeat(Hint::Wrong)
                .take(N)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}
*/

impl<const N: usize> fmt::Display for Hints<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &hint in self.word.iter() {
            let square = match hint {
                Hint::Wrong => "■".red(),
                Hint::OutOfPlace => "■".yellow(),
                Hint::Right => "■".green(),
            };

            write!(f, "{}", square).unwrap();
        }
        writeln!(f, "")
    }
}

impl<const N: usize> TryFrom<Vec<Hint>> for Hints<N> {
    type Error = &'static str;

    fn try_from(value: Vec<Hint>) -> Result<Self, Self::Error> {
        if value.len() != N {
            Err("Wrong size!")
        } else {
            Ok(Self {
                word: value.try_into().unwrap(),
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct WordN<const N: usize> {
    pub word: [char; N],
}

impl<const N: usize> fmt::Display for WordN<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in self.word.iter() {
            write!(f, "{c}").unwrap();
        }
        Ok(())
    }
}

impl<const N: usize> WordN<N> {
    pub fn new(from: &str) -> Self {
        Self {
            word: from.chars().collect::<Vec<_>>().try_into().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct PartialWord<const N: usize> {
    pub word: [Option<char>; N],
}

impl<const N: usize> PartialWord<N> {
    pub fn new() -> Self {
        Self {
            word: iter::repeat(None).take(N).collect::<Vec<_>>().try_into().unwrap(),
        }
    }
}

impl<const N: usize> TryFrom<Vec<Option<char>>> for PartialWord<N> {
    type Error = &'static str;

    fn try_from(value: Vec<Option<char>>) -> Result<Self, Self::Error> {
        if value.len() != N {
            Err("Wrong size!")
        } else {
            Ok(Self {
                word: value.try_into().unwrap(),
            })
        }
    }
}

#[derive(Debug)]
pub struct KnowledgeN<const N: usize> {
    pub known: PartialWord<N>,
    pub out_of_place: Vec<char>,
    pub ruled_out: Vec<char>,
}

impl<const N: usize> KnowledgeN<N> {
    pub fn none() -> Self {
        Self {
            known: PartialWord::<N>::new(),
            out_of_place: vec![],
            ruled_out: vec![],
        }
    }
}
