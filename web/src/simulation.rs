use std::rc::Rc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wordle_entropy_core::{
    algo::{get_answers, get_hints_and_update, update_knowledge},
    entropy::{calculate_entropies, entropies_scored},
};

use crate::{word_set::WordSet, EntropiesData, Knowledge};

#[derive(Clone, Serialize, Deserialize)]
pub enum SimulationInput {
    StartKnownAnswer {
        correct: usize,
        guess: Option<usize>,
    },
    StartUnknownAnswer {
        hints: usize,
        guess: Option<usize>,
    },
    Continue {
        hints: Option<usize>,
        guess: Option<usize>,
    },
    Stop,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationOutput {
    StepComplete {
        guess: usize,
        hints: Option<usize>,
        uncertainty: f64,
        scores: Vec<(usize, EntropiesData, f64)>,
        answers: Vec<usize>,
        knowledge: Knowledge,
    },
    Stopped,
}

pub struct SimulationData {
    word_set: Rc<WordSet>,
    correct: Option<usize>,
    knowledge: Knowledge,
    entropies: Rc<Vec<(usize, EntropiesData, f64)>>,
    answers: Vec<usize>,
}

impl SimulationData {
    fn new(word_set: &Rc<WordSet>, correct: Option<usize>) -> Result<Self> {
        let entropies = word_set
            .entropies
            .as_ref()
            .ok_or(anyhow!("Missing entropies"))?
            .clone();

        let answers = (0..word_set.dictionary.words.len()).collect::<Vec<_>>();

        Ok(Self {
            word_set: word_set.clone(),
            correct,
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
            SimulationInput::StartKnownAnswer { correct, guess } => {
                self.handle_start(word_set, Some(correct), None, guess)
            }
            SimulationInput::StartUnknownAnswer { hints, guess } => {
                self.handle_start(word_set, None, Some(hints), guess)
            }
            SimulationInput::Continue { hints, guess, .. } => self.handle_continue(hints, guess),
            SimulationInput::Stop => self.handle_stop(),
        }
    }

    pub fn handle_start(
        &mut self,
        word_set: &Rc<WordSet>,
        correct: Option<usize>,
        hints: Option<usize>,
        guess: Option<usize>,
    ) -> Result<SimulationOutput> {
        self.state = Some(SimulationData::new(word_set, correct)?);

        self.handle_continue(hints, guess)
    }

    pub fn handle_continue(
        &mut self,
        hints: Option<usize>,
        guess: Option<usize>,
    ) -> Result<SimulationOutput> {
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
        let (hints, knowledge) = match (hints, data.correct) {
            (Some(hints), None) => {
                let guess = &data.word_set.dictionary.words[guess];
                let hints_full = &data.word_set.dictionary.hints[hints];
                let knowledge = update_knowledge(guess, hints_full, data.knowledge.clone());
                (Some(hints), knowledge)
            }
            (None, Some(correct)) => {
                let correct = &data.word_set.dictionary.words[correct];
                let (hints, knowledge) =
                    get_hints_and_update(guess_word, correct, data.knowledge.clone());
                let hints = hints.to_ind();
                (Some(hints), knowledge)
            }
            (Some(_), Some(_)) => {
                return Err(anyhow!(
                    "Tried to pass custom hints to a simulation that already has a known solution"
                ))
            }
            (None, None) => {
                return Err(anyhow!(
                    "Didn't pass custom hints to a simulation that doesn't have a known solution"
                ))
            }
        };

        data.answers = get_answers(data.word_set.dictionary.words.clone(), &knowledge);
        data.knowledge = knowledge.clone();

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
            knowledge,
        })
    }

    pub fn handle_stop(&mut self) -> Result<SimulationOutput> {
        self.state = None;
        Ok(SimulationOutput::Stopped)
    }
}
