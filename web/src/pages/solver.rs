use std::{collections::VecDeque, rc::Rc};

use anyhow::{anyhow, Error, Result};
use bounce::use_atom_setter;
use serde_cbor::ser::to_vec_packed;
use web_sys::{HtmlElement, HtmlInputElement};
use wordle_entropy_core::structs::hints::Hint;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_reducer, use_state, use_state_eq,
    Callback, Event, MouseEvent, Reducible, TargetCast,
};

use crate::components::{HintedWord, SimulationDetail, ToastOption, ToastType};
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::word_set::get_current_word_set;
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::{EntropiesData, Hints, Word};

use super::GuessStep;

fn parse_word(word: &str, words: &Vec<Word>) -> Result<(usize, Word)> {
    let word = Word::try_from(word)?;
    let i = words
        .iter()
        .position(|w| w == &word)
        .ok_or_else(|| anyhow!("Word {word} not found in the current word set!"))?;
    Ok((i, word))
}

enum SolverStateAction {
    NextStep {
        guess: usize,
        hints: usize,
        uncertainty: f64,
        scores: Vec<(usize, EntropiesData, f64)>,
        answers: Vec<usize>,
    },
}

#[derive(Clone, Default, PartialEq)]
struct SolverState {
    history: VecDeque<(usize, Vec<GuessStep>)>,
}

impl Reducible for SolverState {
    type Action = SolverStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SolverStateAction::NextStep {
                guess,
                hints,
                uncertainty,
                scores,
                answers,
            } => {
                let mut history = self.history.clone();

                let mut last_optn: Option<&mut (usize, Vec<GuessStep>)>;
                log::info!("History length: {}", history.len());
                let history_front = if let Some(history_front) = history.front_mut() {
                    history_front
                } else {
                    log::info!("new history");
                    history.push_front((0, vec![]));
                    last_optn = history.front_mut();
                    &mut **last_optn.as_mut().unwrap()
                };

                let scores = scores
                    .into_iter()
                    .map(|(word, entropies_data, left_turns)| {
                        (word, entropies_data.entropy, left_turns)
                    })
                    .collect::<Vec<_>>();
                history_front.1.push(GuessStep {
                    guess,
                    hints,
                    uncertainty,
                    scores: scores.clone(),
                    answers: answers.clone(),
                });

                Rc::new(Self { history })
            }
        }
    }
}

#[function_component(Solver)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let solver_state = use_reducer(|| SolverState::default());
    let set_toast = use_atom_setter::<ToastOption>();
    let word = use_state_eq(|| {
        if let Some(&word) = word_set
            .entropies
            .as_ref()
            .and_then(|entropies| entropies.first().map(|(word, _, _)| word))
        {
            (Some(word), word_set.dictionary.words[word].clone())
        } else if let Some(word) = word_set.dictionary.words.first() {
            (Some(0), word.clone())
        } else {
            (None, Word::try_from("     ").ok().unwrap())
        }
    });
    let word_err = use_state(|| -> Option<Error> { None });

    let hints = use_state_eq(|| Hints::wrong());

    let cb = {
        let set_toast = set_toast.clone();
        let solver_state = solver_state.clone();

        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(_name) => (),
            WordleWorkerOutput::Simulation(output) => match output {
                SimulationOutput::StepComplete {
                    guess,
                    hints: Some(hints),
                    uncertainty,
                    scores,
                    answers,
                    ..
                } => solver_state.dispatch(SolverStateAction::NextStep {
                    guess,
                    hints,
                    uncertainty,
                    scores,
                    answers,
                }),
                _ => set_toast(ToastOption::new(
                    "Hints missing from the step output".to_string(),
                    ToastType::Error,
                )),
            },
            _ => set_toast(ToastOption::new(
                "Unexpected worker output".to_string(),
                ToastType::Error,
            )),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                worker.send(WordleWorkerInput::SetWordSetEncoded(
                    to_vec_packed(&word_set.reduce_entropies(10)).unwrap(),
                ));
                || ()
            },
            word_set_name,
        )
    }

    let onclick_hints = {
        let hints = hints.clone();
        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(i) = element
                .dataset()
                .get("i")
                .and_then(|i| i.parse::<usize>().ok())
            {
                let mut new_hints = (*hints).clone();
                new_hints.0[i] = match hints.0[i] {
                    Hint::Wrong => Hint::OutOfPlace,
                    Hint::OutOfPlace => Hint::Correct,
                    Hint::Correct => Hint::Wrong,
                };
                hints.set(new_hints)
            }
        })
    };

    let onchange_input = {
        let word = word.clone();
        let word_err = word_err.clone();
        let word_set = word_set.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            match parse_word(&input.value(), &word_set.dictionary.words) {
                Ok((i, new_word)) => word.set((Some(i), new_word)),
                Err(err) => word_err.set(err.into()),
            }
        })
    };

    let onclick_enter = {
        let worker = worker.clone();
        let hints = hints.clone();
        let word = word.clone();
        let set_toast = set_toast.clone();

        Callback::from(move |_| {
            if let Some(guess) = word.0 {
                worker.send(WordleWorkerInput::Simulation(
                    SimulationInput::StartUnknownAnswer {
                        hints: hints.to_ind(),
                        guess: Some(guess),
                    },
                ))
            } else {
                set_toast(ToastOption::new(
                    "Guess is not a valid word!".to_string(),
                    ToastType::Error,
                ))
            }
        })
    };

    html! {
        <section>
            <div class="container">
                <div class="columns">
                    <div class="column col-4 col-md-6 col-sm-8 col-xs-10 col-mx-auto">
                        <div class={classes!("form-group", word_err.as_ref().map(|_| "has_error"))}>
                            <label class="form-label">
                            { "Next guess" }
                            </label>
                            <input type="text" placeholder={word.1.to_string()} onchange={onchange_input} />
                            if let Some(ref err) = *word_err {
                                <p class="form-input-hint">{ err }</p>
                            }
                        </div>
                        <div class="form-group">
                            <label class="form-label">
                            { "Hints (click each block to change)" }
                            </label>
                            <div onclick={onclick_hints} class="c-hand">
                                <HintedWord word={word.1.clone()} hints={(*hints).clone()} />
                            </div>
                        </div>
                        <button class="btn btn-primary" onclick={onclick_enter}>{ "Enter" }</button>
                    </div>
                </div>
            </div>
            <SimulationDetail history={solver_state.history.clone()} word_set={word_set.clone()} />
        </section>
    }
}
