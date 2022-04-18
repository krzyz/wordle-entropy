use std::rc::Rc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wordle_entropy_core::{
    algo::{get_answers, get_hints_and_update},
    entropy::{calculate_entropies, entropies_scored},
};

use crate::{word_set::WordSet, EntropiesData, Knowledge};

#[derive(Clone, Serialize, Deserialize)]
pub enum SimulationInput {
    Start(usize, Option<usize>),
    Continue(Option<usize>),
    Stop,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationOutput {
    StepComplete {
        guess: usize,
        hints: usize,
        uncertainty: f64,
        scores: Vec<(usize, EntropiesData, f64)>,
        answers: Vec<usize>,
    },
    Stopped,
}

pub struct SimulationData {
    word_set: Rc<WordSet>,
    correct: usize,
    knowledge: Knowledge,
    entropies: Rc<Vec<(usize, EntropiesData, f64)>>,
    answers: Vec<usize>,
}

impl SimulationData {
    fn new(word_set: &Rc<WordSet>, solution: usize) -> Result<Self> {
        let entropies = word_set
            .entropies
            .as_ref()
            .ok_or(anyhow!("Missing entropies"))?
            .clone();

        let answers = (0..word_set.dictionary.words.len()).collect::<Vec<_>>();

        Ok(Self {
            word_set: word_set.clone(),
            correct: solution,
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
            SimulationInput::Start(correct, guess) => self.handle_start(word_set, correct, guess),
            SimulationInput::Continue(guess) => self.handle_continue(guess),
            SimulationInput::Stop => self.handle_stop(),
        }
    }

    pub fn handle_start(
        &mut self,
        word_set: &Rc<WordSet>,
        correct: usize,
        guess: Option<usize>,
    ) -> Result<SimulationOutput> {
        self.state = Some(SimulationData::new(word_set, correct)?);

        self.handle_continue(guess)
    }

    pub fn handle_continue(&mut self, guess: Option<usize>) -> Result<SimulationOutput> {
        let data = self.state.as_mut().ok_or(anyhow!("Missing state"))?;

        let guess = match guess {
            Some(guess) => guess,
            None => {
                data.entropies
                    .iter()
                    .next()
                    .ok_or(anyhow!(
                        "Neither guess nor entropies available, unable to make the next guess!"
                    ))?
                    .0
            }
        };

        let guess_word = &data.word_set.dictionary.words[guess];
        let correct = &data.word_set.dictionary.words[data.correct];
        let (hints, knowledge) = get_hints_and_update(guess_word, correct, data.knowledge.clone());
        let hints = hints.to_ind();

        data.answers = get_answers(data.word_set.dictionary.words.clone(), &knowledge);
        data.knowledge = knowledge;

        let prob_norm: f64 = data
            .answers
            .iter()
            .map(|&i| &data.word_set.dictionary.probabilities[i])
            .sum();

        let uncertainty = data
            .answers
            .iter()
            .map(|&i| {
                let probability: f64 = &data.word_set.dictionary.probabilities[i] / prob_norm;
                -probability * probability.log2()
            })
            .sum();

        let entropies = calculate_entropies(&data.word_set.dictionary, &data.answers[..]);
        let scores = entropies_scored(
            &data.word_set.dictionary,
            &data.answers[..],
            entropies,
            Some(uncertainty),
            Some(data.word_set.calibration.get_calibration()),
        )
        .into_iter()
        .take(10)
        .collect::<Vec<_>>();
        Ok(SimulationOutput::StepComplete {
            guess,
            hints,
            uncertainty,
            scores,
            answers: data.answers.clone(),
        })
    }

    pub fn handle_stop(&mut self) -> Result<SimulationOutput> {
        self.state = None;
        Ok(SimulationOutput::Stopped)
    }
}
