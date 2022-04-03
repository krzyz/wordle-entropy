use core::fmt;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WordError {
    #[error("Expected word of length: {expected_length}. Found word \"{word}\" of length {}", word.len())]
    IncorrectLength {
        word: String,
        expected_length: usize,
    },
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

impl<const N: usize> TryFrom<&str> for WordN<char, N> {
    type Error = WordError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let array = value
            .chars()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_: Vec<_>| WordError::IncorrectLength {
                word: value.to_string(),
                expected_length: N,
            })?;

        Ok(Self(array))
    }
}
