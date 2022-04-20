use std::{collections::VecDeque, rc::Rc};

use itertools::Itertools;
use yew::{function_component, html, use_state_eq, Html, Properties};

use crate::components::HintedWord;
use crate::{pages::GuessStep, word_set::WordSet};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub history: VecDeque<(usize, Vec<GuessStep>)>,
    pub word_set: Rc<WordSet>,
}

#[function_component(SimulationDetail)]
pub fn view(props: &Props) -> Html {
    let selected = use_state_eq(|| -> Option<usize> { None });
    let word_set = props.word_set.clone();

    let step = if let Some(selected) = *selected {
        props.history.iter().find(|&step| step.0 == selected)
    } else {
        let front = props.history.front();
        if let Some(front) = front {
            if front.1.len() == 0 {
                props.history.iter().nth(1)
            } else {
                Some(front)
            }
        } else {
            None
        }
    };

    if let Some(step) = step {
        let (last_useful_info, ended) =
            if let Some((last, second_to_last)) = step.1.iter().rev().next_tuple() {
                if last.answers.len() == 1 {
                    (Some(second_to_last), true)
                } else {
                    (Some(last), false)
                }
            } else {
                (step.1.iter().last(), false)
            };

        html! {
            <div class="container">
                <div class="columns">
                    <div class="column col-3">
                        <table>
                            <thead>
                                <tr>
                                    <th> { "# Possibilities" } </th>
                                    <th> { "Uncertainty" } </th>
                                </tr>
                            </thead>
                            <tbody>
                            {
                                step.1.iter().map(|GuessStep {uncertainty, ref answers, .. }| {
                                    html! {
                                        <tr>
                                            <td> { format!("{} Pos", answers.len()) } </td>
                                            <td> { format!("{uncertainty:.2} bits") } </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                            </tbody>
                        </table>
                        <ul>
                        {
                            if let Some(GuessStep { ref answers, .. }) = step.1.iter().last() {
                                answers.iter().map(|&answer| {
                                    let probability = word_set.dictionary.probabilities[answer];
                                    let answer = &word_set.dictionary.words[answer];
                                    html! {
                                        <li> { format!("{answer}, {probability:.3}") }  </li>
                                    }
                                }).collect::<Html>()
                            } else {
                                html! { <> </> }
                            }
                        }
                        </ul>
                    </div>
                    <div class="column col-2 col-mx-auto">
                        <table>
                            <thead>
                                <tr>
                                    <th> { "Hints so far" } </th>
                                </tr>
                            </thead>
                            <tbody>
                            {
                                step.1.iter().map(|&GuessStep { guess, hints, .. }| {
                                    let word = &word_set.dictionary.words[guess];
                                    let hints = &word_set.dictionary.hints[hints];
                                    html! {
                                        <tr>
                                            <td>
                                                <HintedWord word={word.clone()} hints={hints.clone()} />
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                            </tbody>
                        </table>
                    </div>
                    <div class="column col-4">
                        <table>
                            <thead>
                                <tr>
                                    <th> { "Top picks" } </th>
                                    <th> { "E[Info.]" } </th>
                                    <th> { "E[Turns]" } </th>
                                </tr>
                            </thead>
                            <tbody>
                                if let Some(GuessStep { ref answers, ref scores, .. }) = last_useful_info {{
                                    scores.iter().map(|&(word_ind, entropy, score)| {
                                        let score = score + step.1.len() as f64 - if ended { 1. } else { 0. };
                                        let word = &word_set.dictionary.words[word_ind];
                                        let possible = answers.contains(&word_ind);
                                        let row = vec![ format!("{word}"), format!("{entropy:.3}"), format!("{score:.3}")].into_iter().map(|text| {
                                            html! {
                                                <td>
                                                    if possible {
                                                        <u> { text } </u>
                                                    } else {
                                                        { text }
                                                    }
                                                </td>
                                            }
                                        }).collect::<Html>();

                                        html! {
                                            <tr>
                                                { row }
                                            </tr>
                                        }
                                    }).collect::<Html>()
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        }
    } else {
        html! { <> </> }
    }
}
