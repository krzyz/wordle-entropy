use bounce::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use yew::Reducible;
use std::rc::Rc;
use wordle_entropy_core::structs::{Dictionary, EntropiesData};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WordSet {
    pub name: String,
    pub dictionary: Rc<Dictionary<5>>,
    pub entropies: Option<Rc<Vec<(EntropiesData<5>, f64)>>>,
}

impl WordSet {
    pub fn from_dictionary(name: String, dictionary: Dictionary<5>) -> Self {
        Self {
            name,
            dictionary: Rc::new(dictionary),
            entropies: None,
        }
    }

    pub fn with_entropies(&self, entropies: Vec<(EntropiesData<5>, f64)>) -> Self {
        Self {
            name: self.name.clone(),
            dictionary: self.dictionary.clone(),
            entropies: Some(Rc::new(entropies)),
        }
    }
}

pub enum WordSetVecAction {
    Set(WordSetVec),
    LoadWords(String, Dictionary<5>),
    SetEntropy(String, Vec<(EntropiesData<5>, f64)>),
}

#[derive(Clone, Debug, PartialEq, Slice, Serialize, Deserialize)]
#[observed]
pub struct WordSetVec(pub Vec<WordSet>);

impl Default for WordSetVec {
    fn default() -> Self {
        WordSetVec(LocalStorage::get("word_set_vec").unwrap_or_else(|_| vec![]))
    }
}

impl WordSetVec {
    pub fn extend_with(&self, word_set: WordSet) -> Self {
        let mut new_vec = self.clone();
        new_vec.0.push(word_set);
        new_vec
    }
}

impl Observed for WordSetVec {
    fn changed(self: Rc<Self>) {
        LocalStorage::set("word_set_vec", (*self).clone()).expect("Failed to set word set collection");
    }
}

impl Reducible for WordSetVec {
    type Action = WordSetVecAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            WordSetVecAction::Set(word_set_vec) => Rc::new(word_set_vec),
            WordSetVecAction::LoadWords(name, dictionary) => { 
                Rc::new(self.extend_with(WordSet::from_dictionary(name, dictionary)))
            }
            WordSetVecAction::SetEntropy(name, entropies_data) => {
                let mut new_vec = self.0.iter().filter(|word_set| word_set.name != name).cloned().collect::<Vec<_>>();
                if let Some(word_set) = self.0.iter().find(|word_set| word_set.name == name).cloned() {
                    new_vec.push(word_set.with_entropies(entropies_data));
                }
                Rc::new(WordSetVec(new_vec))
            } 
        }
    }
}