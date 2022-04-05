use std::rc::Rc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wordle_entropy_core::entropy::calculate_entropies;

use crate::{word_set::WordSet, EntropiesData, Knowledge, Word};

#[derive(Serialize, Deserialize)]
pub enum SimulationInput {
    Start(Word, Option<Word>),
    Continue(Option<Word>),
    Stop,
}

#[derive(Serialize, Deserialize)]
pub enum SimulationOutput {
    StepComplete,
    Finished,
    Stopped,
}

pub struct SimulationData {
    word_set: Rc<WordSet>,
    solution: Word,
    knowledge: Knowledge,
    entropies: Rc<Vec<(EntropiesData, f64)>>,
    answers: Vec<usize>,
}

impl SimulationData {
    fn new(word_set: &Rc<WordSet>, solution: Word) -> Result<Self> {
        let entropies = word_set
            .entropies
            .as_ref()
            .ok_or(anyhow!("Missing entropies"))?
            .clone();

        let answers = (0..word_set.dictionary.words.len()).collect::<Vec<_>>();

        Ok(Self {
            word_set: word_set.clone(),
            solution,
            knowledge: Knowledge::default(),
            entropies,
            answers,
        })
    }
}

#[derive(Default)]
pub struct Simulation {
    state: Option<SimulationData>,
}

impl Simulation {
    pub fn handle_message(
        &mut self,
        word_set: &Rc<WordSet>,
        input: SimulationInput,
    ) -> Result<SimulationOutput> {
        match input {
            SimulationInput::Start(solution, guess) => self.handle_start(word_set, solution, guess),
            SimulationInput::Continue(guess) => self.handle_continue(guess, false),
            SimulationInput::Stop => self.handle_stop(),
        }
    }

    pub fn handle_start(
        &mut self,
        word_set: &Rc<WordSet>,
        solution: Word,
        guess: Option<Word>,
    ) -> Result<SimulationOutput> {
        self.state = Some(SimulationData::new(word_set, solution)?);

        self.handle_continue(guess, true)
    }

    pub fn handle_continue(
        &mut self,
        guess: Option<Word>,
        initial: bool,
    ) -> Result<SimulationOutput> {
        let data = self.state.as_ref().ok_or(anyhow!("Missing state"))?;
        if !initial {
            let entropies = calculate_entropies(&data.word_set.dictionary, &data.answers[..]);
        }
        Ok(SimulationOutput::StepComplete)
    }

    pub fn handle_stop(&mut self) -> Result<SimulationOutput> {
        self.state = None;
        Ok(SimulationOutput::Stopped)
    }
}
