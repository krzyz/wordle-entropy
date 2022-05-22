#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{algo, structs::Dictionary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HintsComputed {
    // Outer vector is indexed by word, w_i
    // Inner vector is indexed by word, w_j
    // Value is hint index
    // Triangular format
    hints_matrix: Vec<Vec<usize>>,
    size: usize,
}

impl HintsComputed {
    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn get_hint(&self, i: usize, j: usize) -> usize {
        assert!(i < self.size());
        assert!(j < self.size());
        let (i, j) = if i <= j { (i, j) } else { (j, i) };

        *self.hints_matrix.get_unchecked(i).get_unchecked(j - i)
    }

    pub fn initialize<const N: usize>(dictionary: &Dictionary<N>) -> Self {
        #[cfg(feature = "parallel")]
        let guess_words_iter = {
            let min_len = dictionary.words.len();
            (0..dictionary.words.len())
                .into_par_iter()
                .with_min_len(min_len)
        };

        #[cfg(not(feature = "parallel"))]
        let guess_words_iter = (0..dictionary.words);

        let hints_matrix = guess_words_iter
            .map(&|i| {
                let guess = &dictionary.words_bytes[i];
                (i..dictionary.words.len())
                    .map(&|j| {
                        let correct = &dictionary.words_bytes[j];
                        algo::get_hints(guess, correct).to_ind()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            hints_matrix,
            size: dictionary.words.len(),
        }
    }
}
