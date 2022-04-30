use std::{collections::VecDeque, rc::Rc};

use anyhow::{anyhow, Result};
use bounce::use_atom_setter;
use itertools::izip;
use serde_cbor::ser::to_vec_packed;
use web_sys::{HtmlElement, HtmlInputElement};
use wordle_entropy_core::algo::get_valid_hints;
use wordle_entropy_core::structs::hints::{Hint, ValidHints};
use wordle_entropy_core::structs::HintsN;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_reducer, Callback, Event,
    MouseEvent, Reducible, TargetCast,
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
use crate::{EntropiesData, Hints, Knowledge, Word, WORD_SIZE};

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
        knowledge: Knowledge,
    },
}

#[derive(Clone, Default, PartialEq)]
struct SolverState {
    history: VecDeque<(usize, Vec<GuessStep>)>,
    knowledge: Knowledge,
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
                knowledge,
            } => {
                let mut history = self.history.clone();

                let mut last_optn: Option<&mut (usize, Vec<GuessStep>)>;
                let history_front = if let Some(history_front) = history.front_mut() {
                    history_front
                } else {
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

                Rc::new(Self { history, knowledge })
            }
        }
    }
}

enum WordStateAction {
    NewWord(String, Rc<WordSet>, Knowledge),
    ToggleHint(usize, Knowledge),
}

#[derive(Clone, PartialEq)]
struct WordState {
    word_ind: Option<usize>,
    word: Word,
    hints: Hints,
    valid_hints: ValidHints,
    error: Option<String>,
}

impl Reducible for WordState {
    type Action = WordStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let WordState {
            mut word_ind,
            mut word,
            mut valid_hints,
            mut hints,
            mut error,
        } = (*self).clone();
        match action {
            WordStateAction::NewWord(new_word, word_set, knowledge) => {
                match parse_word(new_word.as_str(), &word_set.dictionary.words) {
                    Ok((i, new_word)) => {
                        let same_chars = word
                            .0
                            .iter()
                            .zip(new_word.0.iter())
                            .map(|(o, n)| o == n)
                            .collect::<Vec<_>>();

                        word_ind = Some(i);
                        word = new_word;
                        error = None;
                        valid_hints = get_valid_hints(&word, &self.hints, &knowledge);

                        hints = HintsN::<WORD_SIZE>(
                            izip!(&hints.0, &valid_hints.0, same_chars)
                                .map(|(h, valid, same)| {
                                    if valid.contains(h) && same {
                                        *h
                                    } else {
                                        valid.first().copied().unwrap_or(Hint::Wrong)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                        )
                    }
                    Err(err) => {
                        error = Some(err.to_string());
                    }
                }
            }
            WordStateAction::ToggleHint(i, knowledge) => {
                let old_hint = hints.0[i];
                let valid = &valid_hints.0[i];
                let hint_pos = valid.iter().position(|&x| x == old_hint).unwrap_or(0);

                let valid_hints_len = valid.len();

                hints.0[i] = if valid_hints_len > 0 {
                    valid[(hint_pos + 1) % valid_hints_len]
                } else {
                    Hint::Wrong
                };

                valid_hints = get_valid_hints(&self.word, &self.hints, &knowledge);
            }
        }

        Rc::new(Self {
            word_ind,
            word,
            hints,
            valid_hints,
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
            hints: Hints::wrong(),
            valid_hints: ValidHints::any(WORD_SIZE),
            error: None,
        }
    });

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
                    knowledge,
                } => solver_state.dispatch(SolverStateAction::NextStep {
                    guess,
                    hints,
                    uncertainty,
                    scores,
                    answers,
                    knowledge,
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
        let word_state = word_state.clone();
        let solver_state = solver_state.clone();

        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(i) = element
                .dataset()
                .get("i")
                .and_then(|i| i.parse::<usize>().ok())
            {
                word_state.dispatch(WordStateAction::ToggleHint(
                    i,
                    solver_state.knowledge.clone(),
                ));
            }
        })
    };

    let onchange_input = {
        let word_state = word_state.clone();
        let word_set = word_set.clone();
        let solver_state = solver_state.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            word_state.dispatch(WordStateAction::NewWord(
                input.value().clone(),
                word_set.clone(),
                solver_state.knowledge.clone(),
            ));
        })
    };

    let onclick_enter = {
        let worker = worker.clone();
        let word_state = word_state.clone();
        let set_toast = set_toast.clone();
        let solver_state = solver_state.clone();

        Callback::from(move |_| {
            if let Some(guess) = word_state.word_ind {
                if solver_state.history.len() == 0 {
                    worker.send(WordleWorkerInput::Simulation(
                        SimulationInput::StartUnknownAnswer {
                            hints: word_state.hints.to_ind(),
                            guess: Some(guess),
                        },
                    ))
                } else {
                    worker.send(WordleWorkerInput::Simulation(SimulationInput::Continue {
                        hints: Some(word_state.hints.to_ind()),
                        guess: Some(guess),
                    }))
                }
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

    let onclick_suggestion = {
        let word_state = word_state.clone();
        let word_set = word_set.clone();
        let solver_state = solver_state.clone();

        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(word) = element.dataset().get("word") {
                word_state.dispatch(WordStateAction::NewWord(
                    word,
                    word_set.clone(),
                    solver_state.knowledge.clone(),
                ));
            }
        })
    };

    html! {
        <section>
            <div class="container pb-2">
                <div class="columns">
                    <div class="column col-2 col-xl-4 col-sm-6 col-xs-8 col-mx-auto text-center">
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
                                <HintedWord word={word_state.word.clone()} hints={word_state.hints.clone()} />
                            </div>
                        </div>
                        <button class="btn btn-primary mx-1" onclick={onclick_enter}>{ "Enter" }</button>
                        <button class="btn btn-primary mx-1" onclick={onclick_restart}>{ "Restart" }</button>
                    </div>
                </div>
            </div>
            {{
                let (history, init_scores) = if solver_state.history.len() > 0 {
                    (Some(solver_state.history.clone()), None)
                } else if let Some(entropies) = word_set.entropies.as_ref() {
                    let init_scores = scores_without_full_data((**entropies).iter().take(10).cloned().collect::<Vec<_>>());
                    (None, Some(init_scores))
                } else {
                    (None, None)
                };

                if let (None, None) = (history.as_ref(), init_scores.as_ref()) {
                    html! {
                        <div class="text-center">
                            { r#" No word entropies available. Please ensure a word set is loaded and click "Run" on the "Entropy Calculation" page"# }
                        </div>
                    }
                } else {
                    html! {
                        <div onclick={onclick_suggestion} >
                            <SimulationDetail word_set={word_set.clone()} init_scores={init_scores} history={history} />
                        </div>
                    }
                }
            }}
        </section>
    }
}
