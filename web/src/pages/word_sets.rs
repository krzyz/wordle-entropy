use std::rc::Rc;

use anyhow::{anyhow, Result};
use bounce::{use_atom_setter, use_slice, use_slice_dispatch};
use gloo_file::{
    callbacks::{read_as_text, FileReader},
    FileList, FileReadError,
};
use web_sys::{HtmlElement, HtmlInputElement};
use wordle_entropy_core::{calibration::Calibration, data::parse_words};
use yew::{
    function_component, html, use_mut_ref, use_node_ref, Callback, FocusEvent, Html, MouseEvent,
    TargetCast,
};

use crate::{
    components::{ToastOption, ToastType},
    word_set::{SetCalibration, WordSetVec, WordSetVecAction},
    WORD_SIZE,
};

fn handle_file(
    name: String,
    content: std::result::Result<String, FileReadError>,
    dispatch_word_set: Rc<dyn Fn(WordSetVecAction)>,
) -> Result<()> {
    let content = content?;
    let dictionary = parse_words::<_, WORD_SIZE>(content.lines())?;

    dispatch_word_set(WordSetVecAction::LoadWords(name, dictionary));
    Ok(())
}

fn load_from_file(
    name: String,
    files: Option<FileList>,
    dispatch_word_set: Rc<dyn Fn(WordSetVecAction)>,
    set_toast: Rc<dyn Fn(ToastOption)>,
) -> Result<FileReader> {
    if name.is_empty() {
        return Err(anyhow!("Name can't be empty!"));
    }

    let files = files.ok_or(anyhow!("No file selected!"))?;
    let file = files.first().ok_or(anyhow!("No file selected!"))?;

    Ok(read_as_text(&file, move |res| {
        match handle_file(name, res, dispatch_word_set) {
            Ok(_) => (),
            Err(err) => set_toast(ToastOption::new(
                format!("Reading file error: {err}").to_string(),
                ToastType::Error,
            )),
        }
    }))
}

#[function_component(AddWordSetForm)]
pub fn form() -> Html {
    let dispatch_word_set = use_slice_dispatch::<WordSetVec>();
    let file_input_node_ref = use_node_ref();
    let name_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);
    let set_toast = use_atom_setter::<ToastOption>();

    let onload = {
        let file_reader = file_reader.clone();
        let file_input_node_ref = file_input_node_ref.clone();
        let name_input_node_ref = name_input_node_ref.clone();
        let dispatch_word_set = dispatch_word_set.clone();
        let set_toast = set_toast.clone();

        Callback::from(move |e: FocusEvent| {
            let dispatch_word_set = dispatch_word_set.clone();
            let set_toast = set_toast.clone();
            e.prevent_default();
            let name_input = name_input_node_ref.cast::<HtmlInputElement>().unwrap();
            let name = name_input.value();

            let file_input = file_input_node_ref.cast::<HtmlInputElement>().unwrap();
            let files = file_input
                .files()
                .map(|files| gloo_file::FileList::from(files));

            match load_from_file(name, files, dispatch_word_set, set_toast.clone()) {
                Ok(loaded_file_reader) => *file_reader.borrow_mut() = Some(loaded_file_reader),
                Err(err) => set_toast(ToastOption::new(
                    format!("Reading file error: {err}").to_string(),
                    ToastType::Error,
                )),
            }
        })
    };

    html! {
        <form class="form-horizontal" onsubmit={onload}>
            <div class="form-group">
                <div class="col-3">
                    <label class="form-label" for="name_input">{ "Name" }</label>
                </div>
                <div class="col-9">
                    <input id="name_input" class="form-input" ref={name_input_node_ref} />
                </div>
            </div>
            <div class="form-group">
                <div class="col-3">
                    <label class="form-label" for="name_input">{ "File" }</label>
                </div>
                <div class="col-9">
                    <input class="form-input" ref={file_input_node_ref} type="file"/>
                </div>
            </div>
            <div class="form-group">
                <div class="col-8" />
                <div class="col-4">
                    <button class="btn btn-primary">{"Add new words"}</button>
                </div>
            </div>
        </form>
    }
}

#[function_component(WordSets)]
pub fn view() -> Html {
    let word_sets = use_slice::<WordSetVec>();
    let dispatch_word_sets = use_slice_dispatch::<WordSetVec>();

    let onclick_remove = {
        let dispatch_word_sets = dispatch_word_sets.clone();
        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(name) = element.dataset().get("name") {
                dispatch_word_sets(WordSetVecAction::Remove(name));
            }
        })
    };

    let onclick_reset_calibration = {
        let dispatch_word_sets = dispatch_word_sets.clone();
        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(name) = element.dataset().get("name") {
                dispatch_word_sets(WordSetVecAction::SetCalibration(
                    name,
                    SetCalibration::Default,
                ));
            }
        })
    };

    html! {
        <container>
            <h1>
                { "Word sets" }
            </h1>
            <table class="table">
                <thead>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "# of words" }</th>
                        <th>{ "Entropies" }</th>
                        <th>{ "Calibration" }</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {
                        word_sets.0.iter().map(|word_set| {
                            let word_set = word_set;
                            let name = word_set.name.clone();
                            html! {
                                <tr key={name.clone()}>
                                    <td> {name.clone()} </td>
                                    <td> {word_set.dictionary.words.len()} </td>
                                    <td> {
                                        if let Some(_) = word_set.entropies {
                                            html! { <>{ "Available" }</> }
                                        } else {
                                            html! {
                                                <>
                                                    { "Not available" }
                                                </>
                                            }
                                        }
                                    }</td>
                                    <td>
                                    {
                                        match word_set.calibration {
                                            SetCalibration::Default => html! { <> { "Default" } </> },
                                            SetCalibration::Custom(calibration) => {
                                                let Calibration {c, a0, a1 } = calibration;
                                                html! { <>
                                                    { format!("Custom ({c:.3} ln({a0:.3} (x + {a1:.3})))") }
                                                    <button onclick={onclick_reset_calibration.clone()} data-name={name.clone()} class="btn">{"Reset"} </button>
                                                    </> }
                                            }
                                        }
                                    }
                                    </td>
                                    <td>
                                        <button onclick={onclick_remove.clone()} class="btn" data-name={name.clone()}>{"Remove"}</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
            <div class="columns">
                <div class="column col-4 col-xl-3 col-lg-2 col-md-0" />
                <div class="column col-mx text-center">
                    <AddWordSetForm />
                </div>
                <div class="column col-4 col-xl-3 col-lg-2 col-md-0" />
            </div>

        </container>
    }
}
