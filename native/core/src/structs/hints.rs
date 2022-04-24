use std::{iter::repeat, str::FromStr};

#[cfg(feature = "terminal")]
use colored::Colorize;
use core::fmt;
use itertools::{repeat_n, Itertools};
use serde::{
    de::{self, Visitor},
    Deserializer, Serializer,
};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(
    Copy,
    Clone,
    Debug,
    SerializeDisplay,
    DeserializeFromStr,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumIter,
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

    pub fn all() -> Vec<Self> {
        repeat_n(Hint::iter(), N)
            .multi_cartesian_product()
            .map(|hint_vec| Self(hint_vec.try_into().unwrap()))
            .collect()
    }

    pub fn to_ind(&self) -> usize {
        self.0
            .into_iter()
            .enumerate()
            .map(|(i, x)| 3usize.pow((N - i - 1) as u32) * x as usize)
            .sum()
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

#[derive(Debug, Clone)]
pub struct ValidHints(pub Vec<Vec<Hint>>);

impl ValidHints {
    pub fn empty(n: usize) -> Self {
        let vec = repeat(vec![]).take(n).collect();
        Self(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_hints_test() {
        let all_hints = HintsN::<3>::all();
        let expected: Vec<HintsN<3>> = vec![
            "www", "wwo", "wwc", "wow", "woo", "woc", "wcw", "wco", "wcc", "oww", "owo", "owc",
            "oow", "ooo", "ooc", "ocw", "oco", "occ", "cww", "cwo", "cwc", "cow", "coo", "coc",
            "ccw", "cco", "ccc",
        ]
        .into_iter()
        .map(|h| h.parse().unwrap())
        .collect();

        assert_eq!(expected, all_hints);
    }

    #[test]
    fn to_ind_test() {
        let all_hints = HintsN::<4>::all();

        for (i, hints) in all_hints.iter().enumerate() {
            assert_eq!(i, hints.to_ind());
        }
    }
}
