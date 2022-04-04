use std::rc::Rc;

use bounce::use_slice_dispatch;
use web_sys::HtmlInputElement;
use yew::{
    classes, events::Event, function_component, html, use_effect_with_deps, use_state, Callback,
    Html, Reducible, TargetCast,
};
use yew::{use_reducer, UseReducerHandle};

use crate::components::entropy_plot::EntropyPlot;
use crate::word_set::{get_current_word_set, WordSet};
use crate::word_set::{WordSetVec, WordSetVecAction};
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::{EntropiesData, Word};

enum EntropyStateAction {
    ChangeSelected(Option<Word>, Rc<Vec<(EntropiesData, f64)>>, Option<bool>),
    StartRunning,
    StopRunning,
}
#[derive(Clone, PartialEq)]
struct EntropyState {
    running: bool,
    word: Option<Word>,
    data: Vec<f64>,
}

impl EntropyState {
    fn new(word: Word, data: Vec<f64>, running: bool) -> Self {
        Self {
            running,
            word: Some(word),
            data,
        }
    }

    fn empty() -> Self {
        Self {
            running: false,
            word: None,
            data: vec![],
        }
    }

    fn with_running(&self, running: bool) -> Self {
        let rest = (*self).clone();
        Self { running, ..rest }
    }
}

impl Reducible for EntropyState {
    type Action = EntropyStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            EntropyStateAction::ChangeSelected(word, entropies, running) => {
                if let Some(ref word) = word {
                    let word_entropy = entropies
                        .iter()
                        .find(|&(entropies_data, _)| &entropies_data.word == word)
                        .cloned();
                    let data = word_entropy
                        .map(|(entropies_data, _)| {
                            entropies_data
                                .probabilities
                                .into_values()
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or(vec![]);

                    Rc::new(Self::new(
                        word.clone(),
                        data,
                        running.unwrap_or(self.running),
                    ))
                } else {
                    Rc::new(Self::empty())
                }
            }
            EntropyStateAction::StartRunning => Rc::new(self.with_running(true)),
            EntropyStateAction::StopRunning => Rc::new(self.with_running(false)),
        }
    }
}

#[function_component(EntropyCalculation)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());

    let dispatch_word_sets = use_slice_dispatch::<WordSetVec>();
    let selected_state = use_reducer::<EntropyState, _>(|| EntropyState::empty());

    let cb = {
        let selected_state = selected_state.clone();
        let dispatch_word_sets = dispatch_word_sets.clone();
        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(name) => log::info!("Set worker with: {name}"),
            WordleWorkerOutput::Entropy(name, entropies_output) => {
                let entropies = Rc::new(entropies_output);
                let word = entropies
                    .iter()
                    .next()
                    .map(|(entropies_data, _)| Some(entropies_data.word.clone()));
                if let Some(word) = word {
                    selected_state.dispatch(EntropyStateAction::ChangeSelected(
                        word,
                        entropies.clone(),
                        Some(false),
                    ));
                } else {
                    selected_state.dispatch(EntropyStateAction::StopRunning);
                }

                log::info!("Setting entropy: {}", name);
                dispatch_word_sets(WordSetVecAction::SetEntropy(name, entropies));
            }
            WordleWorkerOutput::Err(error) => {
                selected_state.dispatch(EntropyStateAction::StopRunning);
                log::info!("{error}");
            }
            _ => log::info!("Unexpected worker output"),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                log::info!("send set work set");
                worker.send(WordleWorkerInput::SetWordSet(
                    (*word_set).without_entropies(),
                ));
                log::info!("finished send");
                || ()
            },
            word_set_name,
        )
    }

    let onclick_run = {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let selected_state = selected_state.clone();
        Callback::from(move |_| {
            log::info!("run");
            log::info!("found dictionary of: {}", word_set.name);
            worker.send(WordleWorkerInput::Entropy(word_set.name.clone()));
            log::info!("dictionary send");
            selected_state.dispatch(EntropyStateAction::StartRunning);
        })
    };

    let onclick_word = {
        |word: Word, selected_state: UseReducerHandle<EntropyState>, word_set: Rc<WordSet>| {
            Callback::from(move |_| {
                selected_state.dispatch(EntropyStateAction::ChangeSelected(
                    Some(word.clone()),
                    word_set.entropies.clone().unwrap_or(Rc::new(vec![])),
                    None,
                ));
            })
        }
    };

    let max_words_shown = use_state(|| 10);
    let on_max_words_shown_change = {
        let max_words_shown = max_words_shown.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(new_num) = input.value().parse::<usize>().ok() {
                max_words_shown.set(new_num);
            }
        })
    };

    let selected_word_val = selected_state.word.clone();
    let running = selected_state.running;
    let data = selected_state.data.clone();

    html! {
        <div class="container">
            <div class="columns">
                <div class="column col-6 col-mx-auto">
                    <button class="btn btn-primary" disabled={running} onclick={onclick_run}>{"Run"}</button>
                    {
                        if running {
                            html!(<div class="d-inline-block loading p-2"></div>)
                        } else {
                            html!()
                        }
                    }
                </div>
            </div>
            <div class="columns">
                <div class="column">
                    <EntropyPlot {data} />
                </div>
                <div class="column">
                    <label for="max_words_shown_input">{"Max words shown:"}</label>
                    <input id="max_words_shown_input" onchange={on_max_words_shown_change} value={(*max_words_shown).to_string()}/>
                    <ul class="words_entropies_list">
                        {
                            if let Some(ref entropies) = word_set.entropies {
                                entropies
                                    .iter().take(*max_words_shown).map(|(entropy_data, left_turns)| {
                                        let word = &entropy_data.word;
                                        let entropy = &entropy_data.entropy;
                                        html! {
                                            <li
                                                key={format!("{word}")}
                                                class={classes!(
                                                    "c-hand",
                                                    (selected_word_val).clone().map(|selected_word| { *word == selected_word }).map(|is_selected| is_selected.then(|| Some("text-primary")))
                                                )}
                                                onclick={onclick_word(word.clone(), selected_state.clone(), word_set.clone())}
                                            >
                                                {format!("{word}: {entropy}, {left_turns}")}
                                            </li>
                                        }
                                    }).collect::<Html>()
                            } else {
                                html! {<> </>}
                            }
                        }
                    </ul>
                </div>
            </div>
        </div>
    }
}
