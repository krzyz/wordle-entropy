use std::rc::Rc;

use crate::word_set::WordSet;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub last_scores: Vec<(usize, f64, bool)>,
    pub word_set: Rc<WordSet>,
}

#[function_component(SimulationDetail)]
pub fn view(props: &Props) -> Html {
    html! {
    <ul class="words_left_list">
        {
            props.last_scores.iter().map(|&(word, score, could_be_answer)| {
                let word = &props.word_set.dictionary.words[word];
                html! {
                    <li> { format!("{word}: {score} | {could_be_answer:#?}")} </li>
                }
            }).collect::<Html>()
        }
    </ul>}
}
