use std::iter;
use std::{collections::VecDeque, rc::Rc};

use either::Either;
use itertools::Itertools;
use yew::{function_component, html, use_state_eq, Html, Properties};

use crate::components::HintedWord;
use crate::{pages::GuessStep, word_set::WordSet};

pub fn render_suggestions(
    scores: &Vec<(usize, f64, f64)>,
    answers: &Vec<usize>,
    word_set: &WordSet,
    num_answers_before: usize,
) -> Html {
    scores
        .iter()
        .map(|&(word_ind, entropy, score)| {
            //let score = score + step.1.len() as f64 - if ended { 1. } else { 0. };
            let score = score + num_answers_before as f64;
            let word = &word_set.dictionary.words[word_ind];
            let possible = answers.contains(&word_ind);
            let row = vec![
                format!("{word}"),
                format!("{entropy:.3}"),
                format!("{score:.3}"),
            ]
            .into_iter()
            .map(|text| {
                html! {
                    <td data-word={word.to_string()}>
                        if possible {
                            <u data-word={word.to_string()}> { text } </u>
                        } else {
                            { text }
                        }
                    </td>
                }
            })
            .collect::<Html>();

            html! {
                <tr class="c-hand">
                    { row }
                </tr>
            }
        })
        .collect::<Html>()
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub history: Option<VecDeque<(usize, Vec<GuessStep>)>>,
    #[prop_or_default]
    pub init_scores: Option<Vec<(usize, f64, f64)>>,
    pub word_set: Rc<WordSet>,
}

#[function_component(SimulationDetail)]
pub fn view(props: &Props) -> Html {
    let selected = use_state_eq(|| -> Option<usize> { None });
    let word_set = props.word_set.clone();
    let empty_history = VecDeque::default();
    let history = props.history.as_ref().unwrap_or(&empty_history);

    let step = if let Some(selected) = *selected {
        history.iter().find(|&step| step.0 == selected)
    } else {
        let front = history.front();
        if let Some(front) = front {
            if front.1.len() == 0 {
                history.iter().nth(1)
            } else {
                Some(front)
            }
        } else {
            None
        }
    };

    html! {
        <div class="container">
            <div class="columns">
                <div class="column col-2 col-xl-1 col-md-12" />
                <div class="column col-3 col-md-4 col-sm-12 text-center">
                    <table class="centered">
                        <thead>
                            <tr>
                                <th> { "# Possibilities" } </th>
                                <th> { "Uncertainty" } </th>
                            </tr>
                        </thead>
                        <tbody>
                        {
                            if let Some(step) = step {
                                step.1.iter().map(|GuessStep {uncertainty, ref answers, .. }| {
                                    html! {
                                        <tr>
                                            <td> { format!("{} Pos", answers.len()) } </td>
                                            <td> { format!("{uncertainty:.2} bits") } </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            } else {
                                html! {}
                            }
                        }
                        </tbody>
                    </table>
                    <ul>
                    {
                        if let Some(step) = step {
                            if let Some(GuessStep { ref answers, .. }) = step.1.iter().last() {
                                let answers_opt_iter = answers.iter().map(|x| Some(x));
                                let answers_opt_iter = if answers.len() > 10 {
                                    Either::Right(answers_opt_iter.clone().take(5).chain(iter::once(None)).chain(answers_opt_iter.rev().take(5).rev()))
                                } else {
                                    Either::Left(answers_opt_iter)
                                };

                                answers_opt_iter.map(|answer| {
                                    if let Some(&answer) = answer {
                                        let probability = word_set.dictionary.probabilities[answer];
                                        let answer = &word_set.dictionary.words[answer];
                                        html! {
                                            <li> { format!("{answer}, {probability:.3}") }  </li>
                                        }
                                    } else {
                                        html! {
                                            <li> { format!("⋮") }  </li>
                                        }
                                    }
                                }).collect::<Html>()
                            } else {
                                html! {}
                            }
                        } else {
                            html! {}
                        }
                    }
                    </ul>
                </div>
                <div class="column col-2 col-xl-4 col-sm-10 col-mx-auto text-center">
                    <table class="centered">
                        <thead>
                            <tr>
                                <th> { "Hints so far" } </th>
                            </tr>
                        </thead>
                        <tbody>
                        {
                            if let Some(step) = step {
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
                            } else {
                                html! {}
                            }
                        }
                        </tbody>
                    </table>
                </div>
                <div class="column col-3 col-md-4 col-sm-12 text-center">
                    <table class="centered">
                        <thead>
                            <tr>
                                <th> { "Top picks" } </th>
                                <th> { "E[Info.]" } </th>
                                <th> { "E[Turns]" } </th>
                            </tr>
                        </thead>
                        <tbody>
                        {
                            if let Some(step) = step {{{
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

                                if let Some(GuessStep { ref answers, ref scores, .. }) = last_useful_info {{{
                                    let num_answers_before = step.1.len() - if ended { 1 } else { 0 };
                                    render_suggestions(scores, answers, word_set.as_ref(), num_answers_before)
                                }}} else {
                                    html! {}
                                }
                            }}} else if let Some(scores) = props.init_scores.clone() {{{
                                let answers = (0..word_set.dictionary.words.len()).collect::<Vec<_>>();
                                render_suggestions(&scores, &answers, word_set.as_ref(), 0)
                            }}} else {
                                html! {}
                            }
                        }
                        </tbody>
                    </table>
                </div>
                <div class="column col-2 col-xl-1 col-md-12" />
            </div>
        </div>
    }
}
