use std::{collections::VecDeque, rc::Rc};

use anyhow::{anyhow, Result};
use bounce::use_atom_setter;
use serde_cbor::ser::to_vec_packed;
use web_sys::{HtmlElement, HtmlInputElement};
use wordle_entropy_core::structs::hints::Hint;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_reducer, use_state_eq, Callback,
    Event, MouseEvent, Reducible, TargetCast,
};
use yew_router::history::History;
use yew_router::hooks::use_history;

use crate::components::{HintedWord, SimulationDetail, ToastOption, ToastType};
use crate::main_app::Route;
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::util::scores_without_full_data;
use crate::word_set::{get_current_word_set, WordSet};
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

                let scores = scores_without_full_data(scores);

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

enum WordStateAction {
    NewWord(String, Rc<WordSet>),
}

#[derive(Clone, PartialEq)]
struct WordState {
    word_ind: Option<usize>,
    word: Word,
    error: Option<String>,
}

impl Reducible for WordState {
    type Action = WordStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let WordState {
            mut word_ind,
            mut word,
            ..
        } = (*self).clone();
        let error;
        match action {
            WordStateAction::NewWord(new_word, word_set) => {
                match parse_word(new_word.as_str(), &word_set.dictionary.words) {
                    Ok((i, new_word)) => {
                        word_ind = Some(i);
                        word = new_word;
                        error = None;
                    }
                    Err(err) => {
                        error = Some(err.to_string());
                    }
                }
            }
        }

        Rc::new(Self {
            word_ind,
            word,
            error,
        })
    }
}

#[function_component(Solver)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let solver_state = use_reducer(|| SolverState::default());
    let set_toast = use_atom_setter::<ToastOption>();
    let word_state = use_reducer(|| {
        let (word_ind, word) = if let Some(&word) = word_set
            .entropies
            .as_ref()
            .and_then(|entropies| entropies.first().map(|(word, _, _)| word))
        {
            (Some(word), word_set.dictionary.words[word].clone())
        } else if let Some(word) = word_set.dictionary.words.first() {
            (Some(0), word.clone())
        } else {
            (None, Word::try_from("     ").ok().unwrap())
        };
        WordState {
            word_ind,
            word,
            error: None,
        }
    });

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
        let word_state = word_state.clone();
        let word_set = word_set.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            word_state.dispatch(WordStateAction::NewWord(
                input.value().clone(),
                word_set.clone(),
            ));
        })
    };

    let onclick_enter = {
        let worker = worker.clone();
        let hints = hints.clone();
        let word_state = word_state.clone();
        let set_toast = set_toast.clone();

        Callback::from(move |_| {
            if let Some(guess) = word_state.word_ind {
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

    let onclick_restart = {
        let route_history = use_history().unwrap();
        Callback::once(move |_| {
            route_history.push(Route::Solver);
        })
    };

    html! {
        <section>
            <div class="container">
                <div class="columns">
                    <div class="column col-2 col-md-4 col-xs-8 col-mx-auto">
                        <div class={classes!("form-group", word_state.error.as_ref().map(|_| "has-error"))}>
                            <label class="form-label">
                            { "Next guess" }
                            </label>
                            <input type="text" placeholder={word_state.word.to_string()} onchange={onchange_input} />
                            if let Some(ref err) = word_state.error{
                                <p class="form-input-hint">{ err }</p>
                            }
                        </div>
                        <div class="form-group">
                            <label class="form-label">
                            { "Hints (click each block to change)" }
                            </label>
                            <div onclick={onclick_hints} class="c-hand">
                                <HintedWord word={word_state.word.clone()} hints={(*hints).clone()} />
                            </div>
                        </div>
                        <button class="btn btn-primary mx-1" onclick={onclick_enter}>{ "Enter" }</button>
                        <button class="btn btn-primary mx-1" onclick={onclick_restart}>{ "Restart" }</button>
                    </div>
                </div>
            </div>
            {{
                if solver_state.history.len() > 0 {
                    html! { <SimulationDetail history={solver_state.history.clone()} word_set={word_set.clone()} /> }
                } else if let Some(entropies) = word_set.entropies.as_ref() {
                    let init_scores = scores_without_full_data((**entropies).iter().take(10).cloned().collect::<Vec<_>>());
                    html! { <SimulationDetail word_set={word_set.clone()} init_scores={init_scores} /> }
                } else {
                    html! {r#" No word entropies available. Please ensure a word set is loaded and click "Run" on the "Entropy Calculation" page"#}
                }

            }}
        </section>
    }
}
