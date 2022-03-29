use crate::{word_set::{WordSetVec, WordSetVecAction}, main_app::Route};
use bounce::{use_slice_dispatch, use_slice};
use gloo_file::callbacks::read_as_text;
use web_sys::HtmlInputElement;
use wordle_entropy_core::data::parse_words;
use yew::{function_component, html, Html, use_node_ref, Callback, FocusEvent, use_mut_ref};
use yew_router::components::Link;

#[function_component(AddWordSetForm)]
pub fn form() -> Html {
    let dispatch_word_set = use_slice_dispatch::<WordSetVec>();
    let file_input_node_ref = use_node_ref();
    let name_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let onload = {
        let file_reader = file_reader.clone();
        let file_input_node_ref = file_input_node_ref.clone();
        let name_input_node_ref = name_input_node_ref.clone();
        let dispatch_word_set = dispatch_word_set.clone();

        Callback::from(move |e: FocusEvent| {
            let dispatch_word_set = dispatch_word_set.clone();
            e.prevent_default();
            let name_input = name_input_node_ref.cast::<HtmlInputElement>().unwrap();
            let name = name_input.value();
            if name != "" {
                let file_input = file_input_node_ref.cast::<HtmlInputElement>().unwrap();
                let files = file_input
                    .files()
                    .map(|files| gloo_file::FileList::from(files));

                if let Some(files) = files {
                    if let Some(file) = files.first() {
                        *file_reader.borrow_mut() = Some(read_as_text(&file, move |res| match res {
                            Ok(content) => {
                                let dictionary = parse_words::<_, 5>(
                                    content.lines(),
                                );
                                dispatch_word_set(WordSetVecAction::LoadWords(name, dictionary));
                            }
                            Err(err) => {
                                log::info!("Reading file error: {err}");
                            }
                        }));
                    }
                }
            } else {
                log::info!("Name can't be empty!");
            }
        })
    };

    html! {
        <form onsubmit={onload}>
            <label for="name_input">{ "Name" }</label>
            <input id="name_input" ref={name_input_node_ref} />
            <input class="btn" ref={file_input_node_ref} type="file"/>
            <button class="btn btn-primary">{"Add new words"}</button>
        </form>
    }
}

#[function_component(WordSets)]
pub fn view() -> Html {
    let word_sets = use_slice::<WordSetVec>();
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
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {
                        word_sets.0.iter().map(|word_set| {
                            let word_set = word_set;
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
                                                    <span> {"Unloaded("} </span>
                                                    <Link<Route> to={Route::EntropyCalculation} >{"Generate"}</Link<Route>>
                                                    <span> {")"} </span>
                                                </>
                                            }
                                        }
                                    }</td>
                                    <td>
                                        <button class="btn">{"Remove"}</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
            <AddWordSetForm />

        </container>
    }
}
