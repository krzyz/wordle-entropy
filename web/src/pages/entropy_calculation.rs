use std::rc::Rc;

use bounce::{use_atom_setter, use_slice_dispatch};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::{
    classes, events::Event, function_component, html, use_effect_with_deps, use_state, Callback,
    Html, Reducible, TargetCast,
};
use yew::{use_reducer, MouseEvent};

use crate::components::toast::{ToastOption, ToastType};
use crate::components::Plot;
use crate::plots::EntropiesPlotter;
use crate::word_set::get_current_word_set;
use crate::word_set::{WordSetVec, WordSetVecAction};
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::EntropiesData;

enum EntropyStateAction {
    Ready,
    ChangeSelected(
        Option<usize>,
        Rc<Vec<(usize, EntropiesData, f64)>>,
        Option<bool>,
    ),
    StartRunning,
    StopRunning,
}
#[derive(Clone, PartialEq)]
struct EntropyState {
    running: bool,
    ready: bool,
    word: Option<usize>,
    data: Vec<f64>,
}

impl EntropyState {
    fn new(word: usize, data: Vec<f64>, running: bool, ready: bool) -> Self {
        Self {
            running,
            ready: ready,
            word: Some(word),
            data,
        }
    }

    fn empty(ready: bool) -> Self {
        Self {
            running: false,
            ready,
            word: None,
            data: vec![],
        }
    }

    fn with_running(&self, running: bool) -> Self {
        let rest = (*self).clone();
        Self { running, ..rest }
    }

    fn with_ready(&self, ready: bool) -> Self {
        let rest = (*self).clone();
        Self { ready, ..rest }
    }
}

impl Reducible for EntropyState {
    type Action = EntropyStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            EntropyStateAction::Ready => Rc::new(self.with_ready(true)),
            EntropyStateAction::ChangeSelected(word, entropies, running) => {
                if let Some(word) = word {
                    let word_entropy = entropies
                        .iter()
                        .find(|&(entropies_word, _, _)| *entropies_word == word)
                        .cloned();
                    let data = word_entropy
                        .map(|(_, entropies_data, _)| {
                            entropies_data.probabilities.into_iter().collect::<Vec<_>>()
                        })
                        .unwrap_or(vec![]);

                    Rc::new(Self::new(
                        word.clone(),
                        data,
                        running.unwrap_or(self.running),
                        self.ready,
                    ))
                } else {
                    Rc::new(Self::empty(self.ready))
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
    let selected_state = use_reducer::<EntropyState, _>(|| EntropyState::empty(false));
    let set_toast = use_atom_setter::<ToastOption>();

    let cb = {
        let selected_state = selected_state.clone();
        let dispatch_word_sets = dispatch_word_sets.clone();
        let set_toast = set_toast.clone();

        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(_) => selected_state.dispatch(EntropyStateAction::Ready),
            WordleWorkerOutput::Entropy(name, entropies_output) => {
                let entropies = Rc::new(entropies_output);
                let word = entropies.iter().next().map(|&(word, _, _)| Some(word));
                if let Some(word) = word {
                    selected_state.dispatch(EntropyStateAction::ChangeSelected(
                        word,
                        entropies.clone(),
                        Some(false),
                    ));
                } else {
                    selected_state.dispatch(EntropyStateAction::StopRunning);
                }

                dispatch_word_sets(WordSetVecAction::SetEntropy(name, entropies));
            }
            WordleWorkerOutput::Err(err) => {
                selected_state.dispatch(EntropyStateAction::StopRunning);
                set_toast(ToastOption::new(
                    format!("Worker error: {err}").to_string(),
                    ToastType::Error,
                ))
            }
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
                worker.send(WordleWorkerInput::SetWordSet(
                    (*word_set).without_entropies(),
                ));
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
            worker.send(WordleWorkerInput::Entropy(word_set.name.clone()));
            selected_state.dispatch(EntropyStateAction::StartRunning);
        })
    };

    let onclick_word = {
        let word_set = word_set.clone();
        let selected_state = selected_state.clone();
        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(word) = element.dataset().get("word") {
                selected_state.dispatch(EntropyStateAction::ChangeSelected(
                    Some(word.as_str().parse::<usize>().unwrap()),
                    word_set.entropies.clone().unwrap_or(Rc::new(vec![])),
                    None,
                ));
            }
        })
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
    let ready = selected_state.ready;
    let data = selected_state.data.clone();

    html! {
        <div class="container">
            <div class="columns">
                <div class="column col-6 col-mx-auto">
                    <button class="btn btn-primary" disabled={running || !ready} onclick={onclick_run}>{"Run"}</button>
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
                <div class="column col-8 col-xl-12">
                    <Plot<f64, EntropiesPlotter> {data} plotter={EntropiesPlotter{}} />
                </div>
                <div class="column col-4 col-xl-12">
                    <label for="max_words_shown_input">{"Max words shown:"}</label>
                    <input id="max_words_shown_input" onchange={on_max_words_shown_change} value={(*max_words_shown).to_string()}/>
                    <div class="words_entropies_list">
                        <table class="table" onclick={onclick_word}>
                            <thead>
                                <tr>
                                    <th>{"Word"}</th>
                                    <th>{"Exp. Entropy"}</th>
                                    <th>{"Exp. Turns left"}</th>
                                    <th>{"Rel. Probability"}</th>
                                </tr>
                            </thead>
                            <tbody>
                            {
                                if let Some(ref entropies) = word_set.entropies {
                                    entropies
                                        .iter().take(*max_words_shown).map(|(word, entropy_data, left_turns)| {
                                            let entropy = &entropy_data.entropy;
                                            html! {
                                                <tr
                                                    key={format!("{word}")}
                                                    class={classes!(
                                                        "c-hand",
                                                        (selected_word_val).clone().map(|selected_word| { *word == selected_word }).map(|is_selected| is_selected.then(|| Some("text-primary")))
                                                    )}
                                                >
                                                    <td data-word={format!("{word}")}> { &word_set.dictionary.words[*word] }</td>
                                                    <td data-word={format!("{word}")}> { format!("{entropy:.3}") } </td>
                                                    <td data-word={format!("{word}")}> { format!("{left_turns:.3}") } </td>
                                                    <td data-word={format!("{word}")}> { format!("{:.3}", &word_set.dictionary.probabilities[*word]) } </td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                } else {
                                    html! {<> </>}
                                }
                            }
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
