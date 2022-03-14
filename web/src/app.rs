use std::rc::Rc;

use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridged, Worker};
use web_sys::{FocusEvent, HtmlInputElement};
use wordle_entropy_core::data::parse_words;
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};

use crate::worker::WordleWorker;

#[function_component(App)]
pub fn app() -> Html {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = use_state(|| {
        window
            .performance()
            .expect("performance should be available")
    });
    let words = use_state(|| vec![]);
    let perf_start = use_state(|| -> Option<f64> { None });
    let perf_end = use_state(|| -> Option<f64> { None });
    let random_words_num = use_mut_ref(|| 10);
    let file_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let cb = {
        let performance = performance.clone();
        let perf_end= perf_end.clone();
        move |best_scores: <WordleWorker as Worker>::Output| {
            perf_end.set(Some(performance.now()));
            for (word, entropy, score) in best_scores {
                log::info!("{word}: {entropy} entropy, {score} score");
            }
        }
    };

    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));

    let onclick = {
        let words = words.clone();
        let worker = worker.clone();
        let performance = performance.clone();
        let perf_start = perf_start.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            perf_start.set(Some(performance.now()));
            let perf_now = performance.now();
            log::info!("{perf_now}");
            log::info!("{perf_start:#?}");
            worker.borrow_mut().send((*words).clone());
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
            <ul>
                { for words.iter().take(10).map( |x| { html!{<li> { x } </li>} } ) }
            </ul>
            if let (Some(perf_start), Some(perf_end)) = (*perf_start, *perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
        </main>
    }
}
