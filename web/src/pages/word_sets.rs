use crate::{word_set::WordSetVec, main_app::Route};
use bounce::use_atom;
use yew::{function_component, html, Html};
use yew_router::components::Link;

#[function_component(WordSets)]
pub fn view() -> Html {
    let word_sets = use_atom::<WordSetVec>();
    html! {
        <container>
            <h1>
                { "Word sets" }
            </h1>
            <table class="table">
                <thead>
                    <tr>
                        <th>{"Name"}</th>
                        <th>{"# of words"}</th>
                        <th>{ "Entropies" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        word_sets.0.iter().map(|word_set| {
                            let word_set = word_set.borrow();
                            let name = word_set.name.clone();
                            html! {
                                <tr>
                                    <td> {name.clone()} </td>
                                    <td> {word_set.dictionary.words.len()} </td>
                                    <td> {
                                        if let Some(_) = word_set.entropies {
                                            html! { <>{"Loaded"}</> }
                                        } else {
                                            html! {
                                                <>
                                                    <span> {"Unloaded"} </span>
                                                    <Link<Route> to={Route::EntropyCalculation { name }} classes="btn">{"Generate"}</Link<Route>>
                                                </>
                                            }
                                        }
                                    }</td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </container>
    }
}
