use wordle_entropy_core::structs::hints::Hint;
use yew::{classes, function_component, html, Html, Properties};

use crate::{Hints, Word};

fn to_class(hint: &Hint) -> String {
    match hint {
        Hint::Wrong => "hint-wrong",
        Hint::OutOfPlace => "hint-out-of-place",
        Hint::Correct => "hint-correct",
    }
    .to_string()
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub word: Word,
    pub hints: Hints,
}

#[function_component(HintedWord)]
pub fn view(props: &Props) -> Html {
    html! {
        props.word.0.iter().zip(props.hints.0.iter()).enumerate().map(|(i, (c, h))| {
            html! {
                <div data-i={i.to_string()} class={classes!("char-block", to_class(h))}>
                 { c }
                </div>
            }
        }).collect::<Html>()
    }
}
