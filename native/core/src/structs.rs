use colored::Colorize;
use crate::arrays;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    iter,
};


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hint {
    Wrong,
    OutOfPlace,
    Correct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HintsN<const N: usize>(#[serde(with = "arrays")] pub [Hint; N]);

impl<const N: usize> HintsN<N> {
    pub fn from_str(hints_str: &str) -> Result<Self, &'static str> {
        hints_str
            .chars()
            .map(|c| match c.to_ascii_lowercase() {
                'w' => Ok(Hint::Wrong),
                'o' => Ok(Hint::OutOfPlace),
                'c' => Ok(Hint::Correct),
                _ => Err("Wrong character"),
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
    }

    pub fn correct() -> Self {
        Self([Hint::Correct; N])
    }

    pub fn wrong() -> Self {
        Self([Hint::Wrong; N])
    }
}

impl<const N: usize> fmt::Display for HintsN<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &hint in self.0.iter() {
            let square = match hint {
                Hint::Wrong => "■".red(),
                Hint::OutOfPlace => "■".yellow(),
                Hint::Correct => "■".green(),
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
            Ok(Self(value.try_into().unwrap()))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound = "T: Serialize, for<'de2> T: Deserialize<'de2>")]
pub struct WordN<T, const N: usize>(#[serde(with = "arrays")] pub [T; N])
where
    T: Serialize,
    for<'de2> T: Deserialize<'de2>;

impl<T, const N: usize> fmt::Display for WordN<T, N>
where
    T: Display + Serialize,
    for<'de2> T: Deserialize<'de2>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.iter() {
            write!(f, "{c}").unwrap();
        }
        Ok(())
    }
}

impl<T, const N: usize> WordN<T, N>
where
    T: Serialize + Copy,
    for<'de2> T: Deserialize<'de2>,
{
    pub fn init(init_value: T) -> Self {
        Self([init_value; N])
    }
}

impl<const N: usize> WordN<char, N> {
    pub fn new(from: &str) -> Self {
        Self(from.chars().collect::<Vec<_>>().try_into().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
