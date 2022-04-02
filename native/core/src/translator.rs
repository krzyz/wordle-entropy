use crate::structs::WordN;
use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Translator {
    char_to_u8: FxHashMap<char, u8>,
    u8_to_char: Vec<char>,
}

impl Translator {
    pub fn generate<const N: usize>(words: &[WordN<char, N>]) -> Self {
        let mut char_to_u8 = FxHashMap::default();
        let mut u8_to_char = Vec::new();
        let mut u: u8 = 0;

        for word in words {
            for c in word.0 {
                if !char_to_u8.contains_key(&c) {
                    char_to_u8.insert(c, u);
                    u8_to_char.push(c);
                    u += 1;
                }
            }
        }

        Self {
            char_to_u8,
            u8_to_char,
        }
    }

    #[allow(dead_code)]
    pub fn to_chars<const N: usize>(&self, byte_word: &WordN<u8, N>) -> WordN<char, N> {
        let mut word = WordN::init('a');
        for (i, b) in byte_word.0.into_iter().enumerate() {
            word.0[i] = self.u8_to_char[b as usize];
        }
        word
    }

    pub fn to_bytes<const N: usize>(&self, word: &WordN<char, N>) -> WordN<u8, N> {
        let mut byte_word = WordN::init(0);
        for (i, c) in word.0.iter().enumerate() {
            byte_word.0[i] = *self.char_to_u8.get(c).unwrap();
        }
        byte_word
    }
}
