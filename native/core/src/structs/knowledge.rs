use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::HashSet, iter};

use super::WordN;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartialChar {
    None,
    Excluded(HashSet<char>),
    Some(char),
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialWord<const N: usize> {
    #[serde_as(as = "[_; N]")]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeN<const N: usize> {
    pub known: FxHashMap<char, u8>,
    pub ruled_out: HashSet<char>,
    pub placed: PartialWord<N>,
    pub guesses: Vec<WordN<char, N>>,
}

impl<const N: usize> KnowledgeN<N> {
    pub fn none() -> Self {
        Self::default()
    }
}
