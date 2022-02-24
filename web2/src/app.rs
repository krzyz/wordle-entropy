use std::rc::Rc;

use gloo_file::callbacks::read_as_text;
use web_sys::{FocusEvent, HtmlInputElement};
use wordle_entropy_core::{data::parse_words, solvers::solve_random};
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};
use gloo_worker::{Bridged, Worker};

use crate::worker::WordleWorker;


#[function_component(App)]
pub fn app() -> Html {
    let cb = move |res: <WordleWorker as Worker>::Output| {
        log::info!("{res:#?}");
    };
    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));
    let words = use_state(|| vec![]);
    let random_words_num = use_mut_ref(|| 10);
    let file_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let onclick = {
        let random_words_num = random_words_num.clone();
        let words = words.clone();
        let worker = worker.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            worker.borrow_mut().send(((*words).clone(), *random_words_num.borrow()));
        })
    };

    let onchange = {
        let random_words_num = random_words_num.clone();
        Callback::from(move |e: Event| {
            log::info!("logging");
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(new_num) = input.value().parse::<usize>().ok() {
                *random_words_num.borrow_mut() = new_num;
            }
        })
    };

    let onload = {
        let words = words.clone();
        let file_reader = file_reader.clone();
        let file_input_node_ref = file_input_node_ref.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let words = words.clone();
            let file_input = file_input_node_ref.cast::<HtmlInputElement>().unwrap();
            let files = file_input
                .files()
                .map(|files| gloo_file::FileList::from(files));

            log::info!("{files:#?}");

            if let Some(files) = files {
                log::info!("Some files");
                if let Some(file) = files.first() {
                    log::info!("File first");
                    *file_reader.borrow_mut() = Some(read_as_text(&file, move |res| {
                        log::info!("Reading ");
                        match res {
                            Ok(content) => {
                                log::info!("Read file");
                                words.set(parse_words::<_, 5>(content.lines()));
                            }
                            Err(err) => {
                                log::info!("Reading file error: {err}");
                            }
                        }
                    }));
                }
            }
        })
    };

    html! {
        <main>
            <form onsubmit={onload}>
                <input ref={file_input_node_ref} type="file"/>
                <button>{"Load words"}</button>
            </form>
            <input {onchange} value={(&*random_words_num.clone()).borrow().to_string()}/>
            <button {onclick}>{"Run"}</button>
            <div>
                { for words.iter().take(10) }
            </div>
        </main>
    }
}
