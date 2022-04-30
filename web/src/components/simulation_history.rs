use std::{collections::VecDeque, rc::Rc};

use web_sys::HtmlInputElement;
use yew::{function_component, html, use_state_eq, Callback, Event, Html, Properties, TargetCast};

use crate::components::{HintedWord, Plot};
use crate::pages::GuessStep;
use crate::{plots::ExpectedTurnsPlotter, word_set::WordSet};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub history: VecDeque<(usize, Vec<GuessStep>)>,
    pub history_small: Vec<(usize, usize)>,
    pub word_set: Rc<WordSet>,
}

#[function_component(SimulationHistory)]
pub fn view(props: &Props) -> Html {
    let weighted_display = use_state_eq(|| true);
    let on_checkbox_weighted_change = {
        let weighted_display = weighted_display.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            weighted_display.set(input.checked());
        })
    };

    let data = props
        .history_small
        .iter()
        .map(|&(turns, word)| (turns, props.word_set.dictionary.probabilities[word]))
        .collect::<Vec<_>>();
    let plotter = ExpectedTurnsPlotter {
        weighted: *weighted_display,
    };
    html! {
    <div class="container">
        <div class="columns">
            <div class="column col-6 col-xl-12">
                <div class="form-group">
                    <label class="form-switch">
                        <input type="checkbox" onchange={on_checkbox_weighted_change} checked={*weighted_display} />
                        <i class="form-icon"></i> { "Display occurencies count weighted by word probabilities"}
                    </label>
                </div>
                {{

                    html! { <Plot<(usize, f64), ExpectedTurnsPlotter> {data} {plotter} />}
                }}
            </div>
            <div class="history column col-6 col-xl-12">
            {
                props.history.iter().map(|row| {
                    html! {
                        <p class="history">
                            {
                                row.1.iter().map(|GuessStep { guess, hints, .. }| {
                                    let word = props.word_set.dictionary.words[*guess].clone();
                                    let hints = props.word_set.dictionary.hints[*hints].clone();
                                    html! {
                                        <>
                                            <HintedWord {word} {hints} />
                                            <div style="display: inline" class="m-2" />
                                            <div style="display: inline" class="m-2" />
                                        </>
                                    }
                                }).collect::<Html>()
                            }
                        </p>
                    }
                }).collect::<Html>()
            }
            </div>
        </div >
    </div>}
}
