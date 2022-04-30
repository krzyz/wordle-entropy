use anyhow::{anyhow, Result};
use bounce::{use_atom, use_atom_setter, use_slice, Atom, UseSliceHandle};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use wordle_entropy_core::data::parse_words;
use yew::events::Event;
use yew::{function_component, html, use_effect, Callback, Html, TargetCast};

use super::toast::{ToastOption, ToastType};
use crate::word_set::{DefaultWordSets, WordSet, WordSetSpec, WordSetVec, WordSetVecAction};
use crate::WORD_SIZE;

async fn handle_word_set_init(word_sets: UseSliceHandle<WordSetVec>) -> Result<()> {
    let client = reqwest::Client::new();
    let default_word_sets_url = "https://wordle.realcomplexity.com/data/default_word_sets.json";
    let response = client.get(default_word_sets_url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Error loading default word sets list from url: {default_word_sets_url}, request status: {}",
            response.status()
        ));
    }

    let text = response.text().await?;

    let default_word_sets: DefaultWordSets = serde_json::from_str(&text)?;

    let mut loaded_word_sets = vec![];
    loaded_word_sets.reserve(default_word_sets.word_sets.len());

    for WordSetSpec {
        name,
        dictionary_url,
    } in default_word_sets.word_sets.into_iter()
    {
        let response = client.get(&dictionary_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Error loading word sets named: {name} from url: {dictionary_url}, request status: {}",
                response.status()
            ));
        }
        let text = response.text().await?;

        let dictionary = parse_words::<_, WORD_SIZE>(text.lines())?;
        loaded_word_sets.push(WordSet::from_dictionary(name, dictionary));
    }

    word_sets.dispatch(WordSetVecAction::Set(
        (*word_sets).extend_with(loaded_word_sets),
    ));

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
        <select class="form-select" name="word_sets" {onchange}>
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
