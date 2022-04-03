use fxhash::FxHashMap;
use std::{collections::HashSet, iter};

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
    pub known: FxHashMap<char, u8>,
    pub ruled_out: HashSet<char>,
    pub placed: PartialWord<N>,
}

impl<const N: usize> KnowledgeN<N> {
    pub fn none() -> Self {
        Self::default()
    }
}
