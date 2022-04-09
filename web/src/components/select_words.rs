use std::{collections::HashSet, rc::Rc};

use crate::{Dictionary, Word};
use anyhow::{anyhow, Error, Result};
use web_sys::HtmlInputElement;
use wordle_entropy_core::structs::WordError;
use yew::{
    classes, function_component, html, use_mut_ref, use_state, Callback, Event, Properties,
    TargetCast,
};

#[derive(Clone, Debug)]
pub enum SelectedWords {
    Random(usize),
    Custom(Vec<Word>),
}

fn parse_custom_words(words_str: &str, dictionary: &Dictionary) -> Result<Vec<Word>> {
    let words: Vec<_> = words_str
        .split(",")
        .map(|word| word.try_into().map_err(|e: WordError| e.into()))
        .collect::<Result<_>>()?;

    let mut words_unchecked = words.iter().collect::<HashSet<_>>();

    for word in dictionary.words.iter() {
        if words_unchecked.contains(word) {
            words_unchecked.remove(word);
        }
    }

    if words_unchecked.len() > 0 {
        return Err(anyhow!(
            "Words not found in the selected word set: {}",
            words_unchecked
                .iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    Ok(words)
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub dictionary: Rc<Dictionary>,
    pub on_words_set: Callback<SelectedWords>,
}

#[function_component(SelectWords)]
pub fn view(props: &Props) -> Html {
    let num_random = use_mut_ref(|| 10);
    let custom_words = use_mut_ref(|| -> Vec<Word> { vec![] });
    let num_random_err = use_state(|| -> Option<Error> { None });
    let custom_words_err = use_state(|| -> Option<Error> { None });
    let selected = use_mut_ref(|| SelectedWords::Random(*num_random.borrow()));

    let on_num_random_change = {
        let num_random = num_random.clone();
        let num_random_err = num_random_err.clone();
        let selected = selected.clone();
        let on_words_set = props.on_words_set.clone();
        let max_words = props.dictionary.words.len();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            match input
                .value()
                .parse::<usize>()
                .map_err(|e| e.into())
                .and_then(|value| {
                    if value > max_words {
                        Err(anyhow!("There are only {max_words} words to choose from!"))
                    } else {
                        Ok(value)
                    }
                }) {
                Ok(value) => {
                    if let SelectedWords::Random(ref mut num_random) = *selected.borrow_mut() {
                        *num_random = value
                    }
                    on_words_set.emit((*selected.borrow()).clone());
                    *num_random.borrow_mut() = value;
                    num_random_err.set(None);
                }
                Err(err) => num_random_err.set(err.into()),
            }
        })
    };

    let on_word_list_change = {
        let custom_words = custom_words.clone();
        let custom_words_err = custom_words_err.clone();
        let selected = selected.clone();
        let on_words_set = props.on_words_set.clone();
        let dictionary = props.dictionary.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            match parse_custom_words(&input.value(), &*dictionary) {
                Ok(words) => {
                    if let SelectedWords::Custom(ref mut custom_words) = *selected.borrow_mut() {
                        *custom_words = words.clone()
                    }
                    on_words_set.emit((*selected.borrow()).clone());
                    *custom_words.borrow_mut() = words;
                    custom_words_err.set(None);
                }
                Err(err) => custom_words_err.set(err.into()),
            }
        })
    };

    let on_num_random_radio_change = {
        let num_random = num_random.clone();
        let selected = selected.clone();
        let on_words_set = props.on_words_set.clone();
        Callback::from(move |_: Event| {
            *selected.borrow_mut() = SelectedWords::Random(*num_random.borrow());
            on_words_set.emit((*selected.borrow()).clone());
        })
    };

    let on_custom_words_radio_change = {
        let custom_words = custom_words.clone();
        let selected = selected.clone();
        let on_words_set = props.on_words_set.clone();
        Callback::from(move |_: Event| {
            *selected.borrow_mut() = SelectedWords::Custom((*custom_words.borrow()).clone());
            on_words_set.emit((*selected.borrow()).clone());
        })
    };

    let selected = &*selected.borrow();

    html! {
        <>
            <div class={classes!("form-group", num_random_err.as_ref().map(|_| "has-error"))}>
                <label class="form-radio form-inline">
                    <input
                        type="radio"
                        name="select-word-type"
                        value="random"
                        checked={matches!(selected, SelectedWords::Random(_))}
                        onchange={on_num_random_radio_change}
                    />
                    <i class="form-icon" />
                    { "Random: " }
                </label>
                <input class="form-input form-inline" type="text" placeholder="10" onchange={on_num_random_change}/>
                if let Some(ref err) = *num_random_err {
                    <p class="form-input-hint">{ err }</p>
                }
            </div>
            <div class={classes!("form-group", custom_words_err.as_ref().map(|_| "has-error"))}>
                <label class="form-radio">
                    <input
                        type="radio"
                        name="select-word-type"
                        value="custom"
                        checked={matches!(selected, SelectedWords::Custom(_))}
                        onchange={on_custom_words_radio_change}
                    />
                    <i class="form-icon" />
                    { "Custom set:" }
                </label>
                <textarea
                    class="form-input form-inline"
                    id="word_list_textarea"
                    placeholder="Word1,Word2,Word3"
                    onchange={on_word_list_change}
                />
                if let Some(ref err) = *custom_words_err {
                    <p class="form-input-hint">{ err }</p>
                }
            </div>
        </>
    }
}
