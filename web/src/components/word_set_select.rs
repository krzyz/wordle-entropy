use crate::word_set::{WordSet, WordSetVec, WordSetVecAction};
use crate::WORD_SIZE;
use bounce::{use_atom, use_slice, Atom};
use reqwest::StatusCode;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use wordle_entropy_core::data::parse_words;
use yew::events::Event;
use yew::{function_component, html, use_effect, Callback, Html, TargetCast};

#[derive(Default, PartialEq, Atom)]
pub struct WordSetSelection(pub Option<String>);

#[function_component(WordSetSelect)]
pub fn word_set_select() -> Html {
    let word_sets = use_slice::<WordSetVec>();
    let selected = use_atom::<WordSetSelection>();

    {
        let word_sets = word_sets.clone();
        use_effect(move || {
            if word_sets.0.len() == 0 {
                spawn_local(async move {
                    let client = reqwest::Client::new();
                    let response = client
                        .get("https://wordle.realcomplexity.com/data/words-scrabble-with_probs.csv")
                        .send()
                        .await
                        .unwrap();

                    match response.status() {
                        StatusCode::OK => {
                            let text = response.text().await.unwrap();
                            let dictionary = parse_words::<_, WORD_SIZE>(text.lines());
                            word_sets.dispatch(WordSetVecAction::Set((*word_sets).extend_with(
                                WordSet::from_dictionary(
                                    "Polish words scrabble".to_string(),
                                    dictionary.unwrap(),
                                ),
                            )));
                            log::info!("Loaded from url");
                        }
                        _ => log::info!("Error loading csv"),
                    }
                });
            } else {
            }
            || ()
        })
    }

    let onchange = {
        let selected = use_atom::<WordSetSelection>();
        Callback::from(move |e: Event| {
            log::info!("on select change");
            let select: HtmlInputElement = e.target_unchecked_into();
            selected.set(WordSetSelection(Some(select.value().clone())));
        })
    };

    if word_sets.0.len() > 0 && *selected == WordSetSelection(None) {
        log::info!("empty word_sets");
        selected.set(WordSetSelection(Some(
            word_sets.0.iter().next().unwrap().name.clone(),
        )));
    }

    html! {
        <select name="word_sets" {onchange}>
            {
                word_sets.0.iter().map(|word_set| {
                    let name = word_set.name.clone();
                    html! {
                        <option value={name.clone()} selected={selected.0.as_ref() == Some(&word_set.name) }> {name} </option>
                    }
                }).collect::<Html>()
            }
        </select>

    }
}
