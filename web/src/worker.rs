use crate::word_set::WordSet;
use crate::EntropiesData;
use anyhow::{anyhow, Result};
use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering::Equal;
use wordle_entropy_core::entropy::calculate_entropies;
use wordle_entropy_core::solvers::expected_turns;
use serde_cbor::from_slice;

#[derive(Serialize, Deserialize)]
pub enum SimulationInput {}

#[derive(Serialize, Deserialize)]
pub enum WordleWorkerInput {
    SetWordSet(WordSet),
    SetWordSetEncoded(Vec<u8>),
    Entropy(String),
    SimulationInput,
}

#[derive(Serialize, Deserialize)]
pub enum SimulationOutput {}

#[derive(Serialize, Deserialize)]
pub enum WordleWorkerOutput {
    SetWordSet(String),
    Entropy(String, Vec<(EntropiesData, f64)>),
    SimulationOutput,
    Err(String),
}

pub struct WordleWorker {
    link: WorkerLink<Self>,
    word_set: Option<WordSet>,
}

impl WordleWorker {
    fn handle_entropy(&mut self, name: &String) -> Result<WordleWorkerOutput> {
        let word_set = self.word_set.as_ref().ok_or(anyhow!(
            "Worker was not initialized correctly, missing word sets"
        ))?;
        let dictionary = word_set.dictionary.clone();
        let answers = (0..dictionary.words.len()).collect::<Vec<_>>();
        let entropies = calculate_entropies(&dictionary, &answers);

        let uncertainty = (dictionary.words.len() as f64).log2();
        let prob_norm: f64 = answers.iter().map(|&i| dictionary.probabilities[i]).sum();

        let mut scores = entropies
            .into_iter()
            .enumerate()
            .map(|(i, entropies_data)| {
                let prob = if answers.contains(&i) {
                    dictionary.probabilities[i] / prob_norm
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(
                    uncertainty - entropies_data.entropy,
                    0.,
                    1.6369421,
                    -0.029045254,
                ) * (1. - prob);

                (entropies_data, left_diff)
            })
            .collect::<Vec<_>>();

        scores.sort_by(|&(_, score1), &(_, score2)| score1.partial_cmp(&score2).unwrap_or(Equal));

        Ok(WordleWorkerOutput::Entropy(name.clone(), scores))
    }

    fn handle_set(&mut self, word_set: WordSet) -> Result<WordleWorkerOutput> {
        let name = word_set.name.clone();
        self.word_set = Some(word_set);
        Ok(WordleWorkerOutput::SetWordSet(name))
    }

    fn handle_set_encoded(&mut self, word_set: Vec<u8>) -> Result<WordleWorkerOutput> {
        let word_set: WordSet = from_slice(&word_set[..]).unwrap();
        let name = word_set.name.clone();
        self.word_set = Some(word_set);
        Ok(WordleWorkerOutput::SetWordSet(name))
    }

}

impl Worker for WordleWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = WordleWorkerInput;
    type Output = WordleWorkerOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            word_set: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let result = match msg {
            WordleWorkerInput::SetWordSet(word_set) => self.handle_set(word_set),
            WordleWorkerInput::SetWordSetEncoded(word_set) => self.handle_set_encoded(word_set),
            WordleWorkerInput::Entropy(name) => self.handle_entropy(&name),
            _ => Err(anyhow!("unsupported message")),
        };
        let output = match result {
            Ok(output) => output,
            Err(err) => WordleWorkerOutput::Err(err.to_string()),
        };
        self.link.respond(id, output);
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}
