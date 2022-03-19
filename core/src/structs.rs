use colored::Colorize;
use core::fmt;
use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    iter,
};

mod arrays {
    use std::{convert::TryInto, marker::PhantomData};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data = Vec::with_capacity(N);
            for _ in 0..N {
                match (seq.next_element())? {
                    Some(val) => data.push(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            match data.try_into() {
                Ok(arr) => Ok(arr),
                Err(_) => unreachable!(),
            }
        }
    }
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

pub type GuessHints<const N: usize> = FxHashMap<HintsN<N>, f32>;
pub type Entropies<const N: usize> = Vec<(WordN<char, N>, (f32, GuessHints<N>))>;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(bound = "T: Serialize, for<'de2> T: Deserialize<'de2>")]
pub struct WordN<T, const N: usize>(#[serde(with = "arrays")] pub [T; N])
where
    T: Serialize,
    for<'de2> T: Deserialize<'de2>;

impl<T, const N: usize> fmt::Display for WordN<T, N>
where
    T: Display + Serialize + Copy,
    for<'de2> T: Deserialize<'de2>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in self.0.iter() {
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
