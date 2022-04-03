pub mod hints;
pub mod knowledge;
pub mod word;

use crate::translator::Translator;
use fxhash::FxHashMap;
pub use hints::HintsN;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
pub use word::{WordError, WordN};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntropiesData<const N: usize> {
    pub word: WordN<char, N>,
    pub entropy: f64,
    #[serde_as(as = "HashMap<DisplayFromStr, _, _")]
    pub probabilities: FxHashMap<HintsN<N>, f64>,
}

impl<const N: usize> EntropiesData<N> {
    pub fn new(
        word: WordN<char, N>,
        entropy: f64,
        probabilities: FxHashMap<HintsN<N>, f64>,
    ) -> Self {
        EntropiesData {
            word,
            entropy,
            probabilities,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dictionary<const N: usize> {
    pub words: Vec<WordN<char, N>>,
    pub words_bytes: Vec<WordN<u8, N>>,
    pub probabilities: Vec<f64>,
    pub translator: Translator,
}

impl<const N: usize> Dictionary<N> {
    pub fn new(words: Vec<WordN<char, N>>, probabilities: Vec<f64>) -> Self {
        let translator = Translator::generate(&words);
        let words_bytes = words.iter().map(|w| translator.to_bytes(w)).collect();
        Self {
            words,
            words_bytes,
            probabilities,
            translator,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn serialize_entropies_data() {
        let word = "word1".try_into().unwrap();
        let mut hints_map = FxHashMap::default();
        hints_map.insert(HintsN::<5>::from_str("OOWWC").unwrap(), 0.12);
        let entropies_data = EntropiesData::<5>::new(word, 0., hints_map);

        let serialized = serde_json::to_string(&entropies_data).unwrap();
        print!("{}", serialized);
    }
}
