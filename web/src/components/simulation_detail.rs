use std::iter;
use std::{collections::VecDeque, rc::Rc};

use either::Either;
use itertools::Itertools;
use web_sys::HtmlElement;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_state_eq, Callback, Html,
    MouseEvent, Properties, TargetCast,
};

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
                <tr>
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
    #[prop_or_default]
    pub suggestions_clickable: bool,
}

#[function_component(SimulationDetail)]
pub fn view(props: &Props) -> Html {
    let selected_word = use_state_eq(|| -> Option<usize> { None });
    let selected_step = use_state_eq(|| -> Option<usize> { None });
    let word_set = props.word_set.clone();
    let empty_history = VecDeque::default();
    let history = props.history.as_ref().unwrap_or(&empty_history);

    {
        let selected_step = selected_step.clone();

        use_effect_with_deps(
            move |_| {
                selected_step.set(None);
                || ()
            },
            props.history.clone(), // dependents
        );
    }

    let step = if let Some(selected) = *selected_word {
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

    let onclick_step = {
        let selected_step = selected_step.clone();

        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(step) = element
                .dataset()
                .get("step")
                .and_then(|x| x.parse::<usize>().ok())
            {
                selected_step.set(Some(step));
            }
        })
    };

    let c_suggestions_clickable = props.suggestions_clickable.then(|| "c-hand".to_string());

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
                        <tbody class="c-hand" onclick={onclick_step}>
                        {
                            if let Some(step) = step {
                                step.1.iter().enumerate().map(|(i, GuessStep {uncertainty, ref answers, .. })| {
                                    let c_selected = selected_step.filter(|&selected_i| selected_i == i).map(|_| "selected".to_string());

                                    html! {
                                        <tr class={classes![c_selected]}>
                                            <td data-step={i.to_string()}> { format!("{} Pos", answers.len()) } </td>
                                            <td data-step={i.to_string()}> { format!("{uncertainty:.2} bits") } </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            } else {
                                html! {}
                            }
                        }
                        </tbody>
                    </table>
                    <table class="centered">
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
                                            <tr>
                                                <td> { format!("{answer}") } </td>
                                                <td>
                                                    <span class="d-inline-block" style="width: 50px; text-indent: 0px;">
                                                        <progress class="progress ml-2" style="vertical-align: middle;" value={format!("{probability:3}")} max="1" />
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    } else {
                                        html! {
                                            <tr>
                                                <td colspan="2"> { format!("⋮") } </td>
                                            </tr>
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
                    </table>
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
                                step.1.iter().enumerate().map(|(i, &GuessStep { guess, hints, .. })| {
                                    let c_selected = selected_step.filter(|&selected_i| selected_i == i).map(|_| "selected".to_string());

                                    let word = &word_set.dictionary.words[guess];
                                    let hints = &word_set.dictionary.hints[hints];
                                    html! {
                                        <tr class={classes![c_selected]}>
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
                        <tbody class={classes![c_suggestions_clickable]}>
                        {
                            if let Some(step) = step {{{

                                let (info, num_answers_before) =
                                    if let Some(selected_step) = *selected_step {
                                        let info = step.1.iter().nth(selected_step);
                                        let last = selected_step + 1 == step.1.len();
                                        let ended = step.1.last().filter(|x| x.answers.len() == 1).is_some();
                                        let num_answers_before = selected_step +
                                            if last && ended { 0 } else { 1 };
                                        (info, num_answers_before)
                                    } else if let Some((last, second_to_last)) = step.1.iter().rev().next_tuple() {
                                        if last.answers.len() == 1 {
                                            (Some(second_to_last), step.1.len() - 1)
                                        } else {
                                            (Some(last), step.1.len())
                                        }
                                    } else {
                                        (step.1.iter().last(), step.1.len())
                                    };

                                if let Some(GuessStep { ref answers, ref scores, .. }) = info {{{
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
