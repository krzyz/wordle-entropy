use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridge, Bridged, Callback as GlooCallback, Worker};
use std::cmp::Ordering::Equal;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use web_sys::{FocusEvent, HtmlInputElement};
use wordle_entropy_core::data::parse_words;
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};

use crate::worker::WordleWorker;

pub struct WorkerPool<W> {
    workers: Vec<Box<dyn Bridge<W>>>,
    busy_status: Arc<Mutex<Vec<bool>>>,
}

impl<W: Worker + Bridged> WorkerPool<W> {
    pub fn new(num_threads: usize, callback: GlooCallback<W::Output>) -> Self {
        log::info!("new worker poool");
        let busy_status = Arc::new(Mutex::new(vec![false; num_threads]));
        let workers = (0..num_threads)
            .map(|i| {
                let callback_wrapped = {
                    let busy_status = busy_status.clone();
                    let callback = callback.clone();
                    move |output: W::Output| {
                        *busy_status.lock().unwrap().get_mut(i).unwrap() = false;
                        callback(output)
                    }
                };

                W::bridge(Rc::new(callback_wrapped))
            })
            .collect::<Vec<_>>();
        Self {
            workers,
            busy_status,
        }
    }

    pub fn send(&mut self, msg: W::Input) {
        log::info!("Send!");
        let mut busy_status = self.busy_status.lock().unwrap();
        if let Some(i) = busy_status.iter().position(|&x| !x) {
            log::info!("Sending to: {}", i);
            self.workers[i].send(msg);
            *busy_status.get_mut(i).unwrap() = true;
        }
        log::info!("{:#?}", *busy_status)
    }
}

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

    let scores =
        use_mut_ref(|| <WordleWorker as Worker>::Output::new());

    let cb = {
        let performance = performance.clone();
        let perf_end = perf_end.clone();
        let scores = scores.clone();
        move |new_scores: <WordleWorker as Worker>::Output| {
            perf_end.set(Some(performance.now()));

            let mut scores = scores.borrow_mut();
            scores.extend(new_scores);

            scores.sort_by(|&(_, (_, score1, _)), &(_, (_, score2, _))| {
                score1.partial_cmp(&score2).unwrap_or(Equal)
            });
        }
    };
    let cb = Rc::new(cb);
    let worker_pool = use_mut_ref(|| WorkerPool::<WordleWorker>::new(12, cb));

    let onclick = {
        let words = words.clone();
        let worker_pool = worker_pool.clone();
        let performance = performance.clone();
        let perf_start = perf_start.clone();
        let perf_end = perf_end.clone();
        let scores = scores.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            perf_start.set(Some(performance.now()));
            perf_end.set(None);
            *scores.borrow_mut() = vec![];

            let mut words_right = &words[..];
            while words_right.len() > 1000 {
                let (words_left, words_right_new) = words_right.split_at(1000);
                worker_pool.borrow_mut().send((*words_left).to_vec());
                words_right = words_right_new;
            }

            worker_pool.borrow_mut().send((*words_right).to_vec());
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
            <p>
                { words.len() }
            </p>
            if let (Some(perf_start), Some(perf_end)) = (*perf_start, *perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
            <ul>
                { for scores.borrow().iter().take(10).map( |x| { html!{<li> { format!("{}, {}, {}", x.0, x.1.0, x.1.1) } </li>} } ) }
            </ul>
        </main>
    }
}
