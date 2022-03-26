use crate::word_set::WordSetVec;
use yew::{function_component, html, Html, use_context};

#[function_component(WordSets)]
pub fn view() -> Html {
    let word_sets = use_context::<WordSetVec>().expect("no ctx found");
    html! {
        <container>
            <h1>
                { "Word sets" }
            </h1>
            <ul>
            {
                word_sets.iter().map(|word_set| {
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
