pub mod hints;
pub mod knowledge;
pub mod word;

use std::{collections::HashMap, sync::Arc};

use crate::translator::Translator;
pub use hints::HintsN;
use serde::{Deserialize, Serialize};
pub use word::{WordError, WordN};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntropiesData<const N: usize> {
    pub entropy: f64,
    pub probabilities: Vec<f64>,
}

impl<const N: usize> EntropiesData<N> {
    pub fn new(entropy: f64, probabilities: Vec<f64>) -> Self {
        EntropiesData {
            entropy,
            probabilities,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dictionary<const N: usize> {
    pub words: Vec<WordN<char, N>>,
    pub words_bytes: Vec<WordN<u8, N>>,
    pub hints: Vec<HintsN<N>>,
    pub probabilities: Vec<f64>,
    pub translator: Translator,
}

impl<const N: usize> Dictionary<N> {
    pub fn new(words: Vec<WordN<char, N>>, probabilities: Vec<f64>) -> Self {
        let translator = Translator::generate(&words);
        let words_bytes = words.iter().map(|w| translator.to_bytes(w)).collect();
        let hints = HintsN::<N>::all();
        Self {
            words,
            words_bytes,
            hints,
            probabilities,
            translator,
        }
    }
}

pub type EntropiesCacheN<const N: usize> = HashMap<Vec<WordN<char, N>>, Arc<Vec<EntropiesData<N>>>>;
