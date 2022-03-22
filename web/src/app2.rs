use std::rc::Rc;
use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridged, Worker};
use web_sys::{FocusEvent, HtmlInputElement};
use wordle_entropy_core::data::parse_words;
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};
use crate::worker2::DemoWorker;

#[function_component(App2)]
pub fn app() -> Html {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = use_state(|| {
        window
            .performance()
            .expect("performance should be available")
    });
    let perf_start = use_state(|| -> Option<f64> { None });
    let perf_end = use_state(|| -> Option<f64> { None });
    let file_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let cb = {
        let performance = performance.clone();
        let perf_end = perf_end.clone();
        move |word: <DemoWorker as Worker>::Output| {
            perf_end.set(Some(performance.now()));
            log::info!("{word}");
        }
    };


    let words = use_state(|| vec![]);
    let worker = use_mut_ref(|| DemoWorker::bridge(Rc::new(cb)));

    let onclick = {
        let words = words.clone();
        let worker = worker.clone();
        let performance = performance.clone();
        let perf_start = perf_start.clone();
        let perf_end = perf_end.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            perf_start.set(Some(performance.now()));
            perf_end.set(None);
            worker.borrow_mut().send((*words).clone());
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
            <button {onclick}>{"Run"}</button>
            <br />
            if let (Some(perf_start), Some(perf_end)) = (*perf_start, *perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
            <canvas width="700" height="700" />
        </main>
    }
}