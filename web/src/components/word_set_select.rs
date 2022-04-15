use anyhow::{anyhow, Result};
use bounce::{use_atom, use_atom_setter, use_slice, Atom, UseSliceHandle};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use wordle_entropy_core::data::parse_words;
use yew::events::Event;
use yew::{function_component, html, use_effect, Callback, Html, TargetCast};

use super::toast::{ToastOption, ToastType};
use crate::word_set::{WordSet, WordSetVec, WordSetVecAction};
use crate::WORD_SIZE;

async fn handle_word_set_init(word_sets: UseSliceHandle<WordSetVec>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://wordle.realcomplexity.com/data/words-scrabble-with_probs.csv")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Error loading default word sets, request status: {}",
            response.status()
        ));
    }

    let text = response.text().await?;
    let dictionary = parse_words::<_, WORD_SIZE>(text.lines())?;
    word_sets.dispatch(WordSetVecAction::Set((*word_sets).extend_with(
        WordSet::from_dictionary("Polish words scrabble".to_string(), dictionary),
    )));

    Ok(())
}

#[derive(Default, PartialEq, Atom)]
pub struct WordSetSelection(pub Option<String>);

#[function_component(WordSetSelect)]
pub fn word_set_select() -> Html {
    let word_sets = use_slice::<WordSetVec>();
    let selected = use_atom::<WordSetSelection>();
    let set_toast = use_atom_setter::<ToastOption>();

    {
        let word_sets = word_sets.clone();
        let set_toast = set_toast.clone();
        use_effect(move || {
            if word_sets.0.len() == 0 {
                spawn_local(async move {
                    let result = handle_word_set_init(word_sets).await;
                    if let Err(err) = result {
                        set_toast(ToastOption::new(
                            format!("{err}").to_string(),
                            ToastType::Error,
                        ))
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
            let select: HtmlInputElement = e.target_unchecked_into();
            selected.set(WordSetSelection(Some(select.value().clone())));
        })
    };

    if word_sets.0.len() > 0 && *selected == WordSetSelection(None) {
        match word_sets.0.iter().next() {
            Some(word_set) => selected.set(WordSetSelection(Some(word_set.name.clone()))),
            None => set_toast(ToastOption::new(
                "No word sets available".to_string(),
                ToastType::Error,
            )),
        }
    }

    html! {
        <select name="word_sets" {onchange}>
            {
                word_sets.0.iter().map(|word_set| {
                    let name = word_set.name.clone();
                    let is_selected = selected.0.as_ref() == Some(&word_set.name);
                    html! {
                        <option value={name.clone()} selected={is_selected}> {name} </option>
                    }
                }).collect::<Html>()
            }
        </select>

    }
}
