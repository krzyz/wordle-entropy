use crate::translator::Translator;
#[cfg(feature = "terminal")]
use colored::Colorize;
use core::fmt;
use fxhash::FxHashMap;
use serde::{
    de::{self, Visitor},
    Deserializer, Serializer,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeFromStr, SerializeDisplay};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    iter,
    str::FromStr,
};

#[derive(
    Copy, Clone, Debug, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Hint {
    Wrong,
    OutOfPlace,
    Correct,
}

impl FromStr for Hint {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next().map(|x| x.to_ascii_lowercase()) {
            Some(x) if x == 'w' => Ok(Hint::Wrong),
            Some(x) if x == 'o' => Ok(Hint::OutOfPlace),
            Some(x) if x == 'c' => Ok(Hint::Correct),
            _ => Err("Wrong character"),
        }
    }
}

impl fmt::Display for Hint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let char = match self {
            Hint::Wrong => 'w',
            Hint::OutOfPlace => 'o',
            Hint::Correct => 'c',
        };

        write!(f, "{}", char).unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HintsN<const N: usize>(pub [Hint; N]);

impl<const N: usize> Serialize for HintsN<N> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&format!("{}", self))
    }
}

struct HintsNVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for HintsNVisitor<N> {
    type Value = HintsN<N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("hints of length {}", N))
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        HintsN::<N>::from_str(value).map_err(de::Error::custom)
    }
}

impl<'de, const N: usize> Deserialize<'de> for HintsN<N> {
    fn deserialize<D>(deserializer: D) -> Result<HintsN<N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HintsNVisitor::<N>)
    }
}

impl<const N: usize> HintsN<N> {
    pub fn correct() -> Self {
        Self([Hint::Correct; N])
    }

    pub fn wrong() -> Self {
        Self([Hint::Wrong; N])
    }
}

impl<const N: usize> FromStr for HintsN<N> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .map(|c| match c.to_ascii_lowercase() {
                'w' => Ok(Hint::Wrong),
                'o' => Ok(Hint::OutOfPlace),
                'c' => Ok(Hint::Correct),
                _ => Err("Wrong character"),
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
    }
}

#[cfg(feature = "terminal")]
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

#[cfg(not(feature = "terminal"))]
impl<const N: usize> fmt::Display for HintsN<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &hint in self.0.iter() {
            let square = match hint {
                Hint::Wrong => "W",
                Hint::OutOfPlace => "O",
                Hint::Correct => "C",
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound = "T: Serialize, for<'de2> T: Deserialize<'de2>")]
pub struct WordN<T, const N: usize>(#[serde_as(as = "[_; N]")] pub [T; N])
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
    use super::*;

    #[test]
    fn serialize_entropies_data() {
        let word = WordN::<char, 5>::new("word1");
        let mut hints_map = FxHashMap::default();
        hints_map.insert(HintsN::<5>::from_str("OOWWC").unwrap(), 0.12);
        let entropies_data = EntropiesData::<5>::new(word, 0., hints_map);

        let serialized = serde_json::to_string(&entropies_data).unwrap();
        print!("{}", serialized);
    }
}
