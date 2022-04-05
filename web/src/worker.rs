use crate::simulation::{Simulation, SimulationInput, SimulationOutput};
use crate::word_set::WordSet;
use crate::EntropiesData;
use anyhow::{anyhow, Result};
use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use serde::{Deserialize, Serialize};
use serde_cbor::from_slice;
use std::rc::Rc;
use wordle_entropy_core::entropy::{calculate_entropies, entropies_scored};

#[derive(Serialize, Deserialize)]
pub enum WordleWorkerInput {
    CheckEntropies,
    SetWordSet(WordSet),
    SetWordSetEncoded(Vec<u8>),
    Entropy(String),
    Simulation(SimulationInput),
}

#[derive(Serialize, Deserialize)]
pub enum WordleWorkerOutput {
    EntropyState {
        name: Option<String>,
        entropies: bool,
    },
    SetWordSet(String),
    Entropy(String, Vec<(EntropiesData, f64)>),
    Simulation(SimulationOutput),
    Err(String),
}

pub struct WordleWorker {
    link: WorkerLink<Self>,
    word_set: Option<Rc<WordSet>>,
    simulation: Simulation,
}

impl WordleWorker {
    fn handle_entropy(&mut self, name: &String) -> Result<WordleWorkerOutput> {
        let word_set = self.word_set.as_ref().ok_or(anyhow!(
            "Worker was not initialized correctly, missing word sets"
        ))?;
        let dictionary = word_set.dictionary.clone();
        let answers = (0..dictionary.words.len()).collect::<Vec<_>>();
        let entropies = calculate_entropies(&dictionary, &answers);

        let scores = entropies_scored(&dictionary, &answers, entropies);

        Ok(WordleWorkerOutput::Entropy(name.clone(), scores))
    }

    fn handle_set(&mut self, word_set: WordSet) -> Result<WordleWorkerOutput> {
        let name = word_set.name.clone();
        self.word_set = Some(Rc::new(word_set));
        Ok(WordleWorkerOutput::SetWordSet(name))
    }

    fn handle_set_encoded(&mut self, word_set: Vec<u8>) -> Result<WordleWorkerOutput> {
        let word_set: WordSet = from_slice(&word_set[..]).unwrap();
        let name = word_set.name.clone();
        self.word_set = Some(Rc::new(word_set));
        Ok(WordleWorkerOutput::SetWordSet(name))
    }

    fn handle_check_entropies(&mut self) -> Result<WordleWorkerOutput> {
        let name = self.word_set.as_ref().map(|w| w.name.clone());
        let entropies = self
            .word_set
            .as_ref()
            .map(|w| w.entropies.is_some())
            .is_some();
        Ok(WordleWorkerOutput::EntropyState { name, entropies })
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
            simulation: Simulation::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let result = match msg {
            WordleWorkerInput::CheckEntropies => self.handle_check_entropies(),
            WordleWorkerInput::SetWordSet(word_set) => self.handle_set(word_set),
            WordleWorkerInput::SetWordSetEncoded(word_set) => self.handle_set_encoded(word_set),
            WordleWorkerInput::Entropy(name) => self.handle_entropy(&name),
            WordleWorkerInput::Simulation(input) => {
                if let Some(word_set) = self.word_set.as_ref() {
                    self.simulation
                        .handle_message(&word_set, input)
                        .map(|output| WordleWorkerOutput::Simulation(output))
                } else {
                    Err(anyhow!(
                        "Tried to start simulation, but word set is not set".to_string()
                    ))
                }
            }
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
