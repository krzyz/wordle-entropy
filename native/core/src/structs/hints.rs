#[cfg(feature = "terminal")]
use colored::Colorize;
use core::fmt;
use serde::{
    de::{self, Visitor},
    Deserializer, Serializer,
};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

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
