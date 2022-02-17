use colored::Colorize;
use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    iter,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hint {
    Wrong,
    OutOfPlace,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HintsN<const N: usize> {
    pub word: [Hint; N],
}

impl<const N: usize> HintsN<N> {
    pub fn correct() -> HintsN<N> {
        HintsN {
            word: iter::repeat(Hint::Right)
                .take(N)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}
/*
impl<const N: usize> Hints<N> {
    pub fn none() -> Hints<N> {
        HintsN {
            word: iter::repeat(Hint::Wrong)
                .take(N)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}
*/

impl<const N: usize> fmt::Display for HintsN<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &hint in self.word.iter() {
            let square = match hint {
                Hint::Wrong => "■".red(),
                Hint::OutOfPlace => "■".yellow(),
                Hint::Right => "■".green(),
            };

            write!(f, "{}", square).unwrap();
        }
        Ok(())
    }
}

impl<const N: usize> TryFrom<Vec<Hint>> for HintsN<N> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub enum PartialChar {
    None,
    Excluded(HashSet<char>),
    Some(char),
}

#[derive(Debug, Clone)]
pub struct PartialWord<const N: usize> {
    pub word: [PartialChar; N],
}

impl<const N: usize> Default for PartialWord<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PartialWord<N> {
    pub fn new() -> Self {
        Self {
            word: iter::repeat(PartialChar::None)
                .take(N)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}

impl<const N: usize> TryFrom<Vec<PartialChar>> for PartialWord<N> {
    type Error = &'static str;

    fn try_from(value: Vec<PartialChar>) -> Result<Self, Self::Error> {
        if value.len() != N {
            Err("Wrong size!")
        } else {
            Ok(Self {
                word: value.try_into().unwrap(),
            })
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct KnowledgeN<const N: usize> {
    pub known: HashMap<char, u8>,
    pub ruled_out: HashSet<char>,
    pub placed: PartialWord<N>,
}

impl<const N: usize> KnowledgeN<N> {
    pub fn none() -> Self {
        Self::default()
    }
}
