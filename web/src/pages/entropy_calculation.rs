use crate::components::entropy_plot::EntropyPlot;
use crate::word_set::get_current_word_set;
use crate::word_set::{WordSetVec, WordSetVecAction};
use crate::worker::WordleWorker;
use bounce::use_slice_dispatch;
use gloo_worker::{Bridged, Worker};
use yew::UseStateHandle;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use wordle_entropy_core::structs::WordN;
use yew::{
    classes, events::Event, function_component, html, use_mut_ref, use_state, Callback, Html,
    TargetCast
};

#[function_component(EntropyCalculation)]
pub fn view() -> Html {
    let word_set = get_current_word_set();
    let dispatch_word_sets = use_slice_dispatch::<WordSetVec>();
    let selected_word = use_mut_ref(|| -> Option<WordN<char, 5>> { None });
    let running = use_state(|| false);
    let rerender = use_state(|| ());

    let cb = {
        let selected_word = selected_word.clone();
        let dispatch_word_sets = dispatch_word_sets.clone();
        let running = running.clone();
        move |(name, entropies_output): <WordleWorker as Worker>::Output| {
            running.set(false);

            if let Some((entropies_data, _)) = entropies_output.iter().next() {
                *selected_word.borrow_mut() = Some(entropies_data.word.clone());
            }

            log::info!("Setting entropy: {}", name);
            dispatch_word_sets(WordSetVecAction::SetEntropy(name, entropies_output));
        }
    };

    let data = selected_word
        .borrow()
        .as_ref()
        .map(|selected_word| {
            let word_entropy = if let Some(entropies) = word_set.entropies.clone() {
                entropies
                    .iter()
                    .find(|&(entropies_data, _)| &entropies_data.word == selected_word)
                    .cloned()
            } else {
                None
            };
            word_entropy.map(|(entropies_data, _)| {
                entropies_data
                    .probabilities
                    .into_values()
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .unwrap_or(vec![]);

    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));

    let onclick_run = {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let running = running.clone();
        Callback::from(move |_| {
            log::info!("run");
            log::info!("found dictionary of: {}", word_set.name);
            worker
                .borrow_mut()
                .send((word_set.name.clone(), word_set.dictionary.clone()));
            log::info!("dictionary send");
            running.set(true);
        })
    };

    let onclick_word = {
        |word: WordN<_, 5>, selected_word: Rc<RefCell<Option<WordN<_, 5>>>>, rerender: UseStateHandle<()>| {
            Callback::from(move |_| {
                *selected_word.borrow_mut() = Some(word.clone());
                rerender.set(());
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

    let selected_word_val = selected_word.borrow().clone();

    html! {
        <div class="container">
            <div class="columns">
                <div class="column col-6 col-mx-auto">
                    <button class="btn btn-primary" disabled={*running} onclick={onclick_run}>{"Run"}</button>
                    {
                        if *running {
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
                                                onclick={onclick_word(word.clone(), selected_word.clone(), rerender.clone())}
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
