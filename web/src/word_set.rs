use crate::components::word_set_select::WordSetSelection;
use crate::{Dictionary, EntropiesData};
use bounce::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WordSet {
    pub name: String,
    pub dictionary: Rc<Dictionary>,
    pub entropies: Option<Rc<Vec<(EntropiesData, f64)>>>,
}

impl WordSet {
    pub fn from_dictionary(name: String, dictionary: Dictionary) -> Self {
        Self {
            name,
            dictionary: Rc::new(dictionary),
            entropies: None,
        }
    }

    pub fn with_entropies(&self, entropies: Vec<(EntropiesData, f64)>) -> Self {
        Self {
            name: self.name.clone(),
            dictionary: self.dictionary.clone(),
            entropies: Some(Rc::new(entropies)),
        }
    }
}

pub enum WordSetVecAction {
    Set(WordSetVec),
    Remove(String),
    LoadWords(String, Dictionary),
    SetEntropy(String, Vec<(EntropiesData, f64)>),
}

#[derive(Clone, Debug, PartialEq, Slice, Serialize, Deserialize)]
#[observed]
pub struct WordSetVec(pub Vec<WordSet>);

static STORAGE_VEC_NAMES: &str = "word_sets_names";

fn get_word_set_storage_key(name: &str) -> String {
    format!("word_set:{name}")
}

impl Default for WordSetVec {
    fn default() -> Self {
        let names: Option<Vec<String>> = LocalStorage::get(STORAGE_VEC_NAMES).ok();
        let vec = match names {
            Some(names) => names
                .iter()
                .filter_map(|name| LocalStorage::get(get_word_set_storage_key(name)).ok())
                .collect(),
            None => vec![],
        };

        WordSetVec(vec)
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
        let names = self.0.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
        LocalStorage::set(STORAGE_VEC_NAMES, names).ok();
        for word_set_without_entropies in self.0.iter().map(|word_set| WordSet {
            name: word_set.name.clone(),
            dictionary: word_set.dictionary.clone(),
            entropies: None,
        }) {
            LocalStorage::set(
                get_word_set_storage_key(&word_set_without_entropies.name),
                word_set_without_entropies,
            )
            .expect("Failed to set word set collection");
        }
    }
}

impl Reducible for WordSetVec {
    type Action = WordSetVecAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            WordSetVecAction::Set(word_set_vec) => Rc::new(word_set_vec),
            WordSetVecAction::Remove(name) => Rc::new(WordSetVec(
                self.0
                    .iter()
                    .filter(|word_set| word_set.name != name)
                    .cloned()
                    .collect(),
            )),
            WordSetVecAction::LoadWords(name, dictionary) => {
                Rc::new(self.extend_with(WordSet::from_dictionary(name, dictionary)))
            }
            WordSetVecAction::SetEntropy(name, entropies_data) => {
                let mut new_vec = self
                    .0
                    .iter()
                    .filter(|word_set| word_set.name != name)
                    .cloned()
                    .collect::<Vec<_>>();
                if let Some(word_set) = self
                    .0
                    .iter()
                    .find(|word_set| word_set.name == name)
                    .cloned()
                {
                    new_vec.push(word_set.with_entropies(entropies_data));
                }
                Rc::new(WordSetVec(new_vec))
            }
        }
    }
}

pub fn get_current_word_set() -> WordSet {
    let word_sets = use_slice::<WordSetVec>();
    let select = use_atom::<WordSetSelection>();
    let word_set = word_sets
        .0
        .iter()
        .find(|word_set| Some(&word_set.name) == select.0.as_ref())
        .cloned()
        .unwrap_or(WordSet::from_dictionary(
            "invalid_word_set".to_string(),
            Dictionary::new(vec![], vec![]),
        ));

    log::info!("Word set: {}", word_set.name);
    word_set
}
