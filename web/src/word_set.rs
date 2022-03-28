use std::{cell::RefCell, rc::Rc};

use bounce::Atom;
use wordle_entropy_core::structs::{Dictionary, EntropiesData};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WordSetState {
    Unloaded,
    LoadedDictionary,
    LoadedAll,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WordSet {
    pub id: usize,
    pub name: String,
    pub state: WordSetState,
    pub dictionary: Option<Dictionary<5>>,
    pub entropies: Option<Vec<EntropiesData<5>>>,
}

impl WordSet {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            state: WordSetState::Unloaded,
            dictionary: None,
            entropies: None,
        }
    }

    pub fn from_dictionary(id: usize, name: String, dictionary: Dictionary<5>) -> Self {
        Self {
            id,
            name,
            state: WordSetState::LoadedDictionary,
            dictionary: Some(dictionary),
            entropies: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Atom)]
pub struct WordSetVec(pub Vec<Rc<RefCell<WordSet>>>);

impl Default for WordSetVec {
    fn default() -> Self {
        Self(vec![])
    }
}

impl WordSetVec {
    pub fn extend_with(&self, word_set: WordSet) -> Self {
        let mut new_vec = self.clone();
        new_vec.0.push(Rc::new(RefCell::new(word_set)));
        new_vec
    }
}
