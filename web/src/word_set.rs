use std::{rc::Rc, cell::RefCell};

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
}

pub type WordSetVec = Vec<Rc<RefCell<WordSet>>>;