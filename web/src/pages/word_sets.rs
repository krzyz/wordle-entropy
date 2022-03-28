use crate::word_set::WordSetVec;
use bounce::use_atom;
use yew::{function_component, html, Html};

#[function_component(WordSets)]
pub fn view() -> Html {
    let word_sets = use_atom::<WordSetVec>();
    html! {
        <container>
            <h1>
                { "Word sets" }
            </h1>
            <ul>
            {
                word_sets.0.iter().map(|word_set| {
                    let name = word_set.borrow().name.clone();
                    html! {
                        <li> {name} </li>
                    }
                }).collect::<Html>()
            }
            </ul>
        </container>
    }
}
