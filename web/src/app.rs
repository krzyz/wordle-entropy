use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridge, Bridged, Callback as GlooCallback, Worker};
use std::cmp::Ordering::Equal;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Weak};
use parking_lot::Mutex;
use web_sys::{FocusEvent, HtmlInputElement};
use wordle_entropy_core::data::parse_words;
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};

use crate::worker::WordleWorker;

pub struct Link<W: Worker> {
    worker_pool: Weak<Mutex<WorkerPool<W>>>,
}

impl <W: Worker> Link<W> {
    pub fn new() -> Self {
        Self {
            worker_pool: Weak::new(),
        }
    }

    pub fn set_pool(&mut self, pool: &Arc<Mutex<WorkerPool<W>>>) {
        self.worker_pool = Arc::downgrade(pool)
    }
}

impl <W: Worker + Bridged> Link<W> {
    pub fn send(&mut self, msg: W::Input) {
        if let Some(worker_pool) = self.worker_pool.upgrade() {
            log::info!("lock worker pool from link send");
            worker_pool.lock().send(msg);
            log::info!("unlock worker pool from link send");
        }
    }
}

pub struct WorkerPool<W: Worker> {
    workers: Vec<Box<dyn Bridge<W>>>,
    busy_status: Arc<Mutex<Vec<bool>>>,
    jobs_queue: Arc<Mutex<VecDeque<W::Input>>>,
    link: Arc<Mutex<Link<W>>>,
}

impl<W: Worker + Bridged> WorkerPool<W> {
    pub fn new(num_threads: usize, callback: GlooCallback<W::Output>) -> Arc<Mutex<Self>> {
        log::info!("new worker pool");
        let busy_status = Arc::new(Mutex::new(vec![false; num_threads]));
        let jobs_queue = Arc::new(Mutex::new(VecDeque::new()));

        let pool = Arc::new(Mutex::new(Self {
            workers: vec![],
            busy_status,
            jobs_queue,
            link: Arc::new(Mutex::new(Link::new())),
        }));

        pool.lock().link.lock().set_pool(&pool);

        let workers = (0..num_threads)
            .map(|i| {
                let callback_wrapped = {
                    let pool = pool.lock();
                    let busy_status = pool.busy_status.clone();
                    let jobs_queue = pool.jobs_queue.clone();
                    let link = pool.link.clone();
                    let callback = callback.clone();
                    move |output: W::Output| {
                        log::info!("lock jobs_queue from {i}");
                        let mut jobs_queue = jobs_queue.lock();
                        log::info!("lock busy_status from {i}");
                        *busy_status.lock().get_mut(i).unwrap() = false;
                        log::info!("unlock busy_status from {i}");
                        if let Some(msg) = jobs_queue.pop_front() {
                            link.lock().send(msg);
                        } else if !busy_status.lock().iter().any(|&x| x) {
                            log::info!("Finished all!");
                        }
                        log::info!("unlock busy_status from {i}");
                        log::info!("Unlock jobs_queue from {i}");
                        callback(output)
                    }
                };

                W::bridge(Rc::new(callback_wrapped))
            })
            .collect::<Vec<_>>();

        pool.lock().workers.extend(workers);

        pool
    }

    pub fn send(&mut self, msg: W::Input) {
        log::info!("Send!");
        log::info!("Lock busy_status from send");
        let mut busy_status = self.busy_status.lock();
        if let Some(i) = busy_status.iter().position(|&x| !x) {
            log::info!("Sending to: {}", i);
            self.workers[i].send(msg);
            *busy_status.get_mut(i).unwrap() = true;
        } else {
            log::info!("Lock jobs_queue from send");
            let mut jobs_queue = self.jobs_queue.lock();
            jobs_queue.push_back(msg);
            log::info!("Unlock jobs_queue from send");
        }
        log::info!("Unlock busy_status from send");
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
    let chunk_size = use_mut_ref(|| 1000);
    let file_input_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let scores = use_mut_ref(|| <WordleWorker as Worker>::Output::new());
    let running = use_mut_ref(|| false);

    let cb = {
        let performance = performance.clone();
        let perf_end = perf_end.clone();
        let scores = scores.clone();
        move |new_scores: <WordleWorker as Worker>::Output| {
            {
                let mut scores = scores.borrow_mut();
                scores.extend(new_scores);

                scores.sort_by(|&(_, (_, score1, _)), &(_, (_, score2, _))| {
                    score1.partial_cmp(&score2).unwrap_or(Equal)
                });
            }
            perf_end.set(Some(performance.now()));
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
        let chunk_size = chunk_size.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            perf_start.set(Some(performance.now()));
            perf_end.set(None);
            *running.borrow_mut() = true;
            *scores.borrow_mut() = vec![];

            let mut words_right = &words[..];
            let chunk_size = *chunk_size.borrow();
            while words_right.len() > chunk_size {
                let (words_left, words_right_new) = words_right.split_at(chunk_size);
                log::info!("Lock worker_pool");
                worker_pool.borrow_mut().lock().send(((*words_left).to_vec(), (*words).clone()));
                words_right = words_right_new;
                log::info!("Unlock worker_pool");
            }

            log::info!("Lock worker_pool");
            worker_pool.borrow_mut().lock().send(((*words_right).to_vec(), (*words).clone()));
            log::info!("Unlock worker_pool");
        })
    };

    let onchange = {
        let chunk_size = chunk_size.clone();
        Callback::from(move |e: Event| {
            log::info!("logging");
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(new_num) = input.value().parse::<usize>().ok() {
                *chunk_size.borrow_mut() = new_num;
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
            <input {onchange} value={(&*chunk_size.clone()).borrow().to_string()}/>
            <button {onclick}>{"Run"}</button>
            <p>
                { words.len() }
            </p>
            if let (Some(perf_start), Some(perf_end)) = (*perf_start, *perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
            if words.len() > 0 {
                <p>
                    { scores.borrow().len() as f32 / words.len() as f32 }
                </p>
                if let Some((word, (entropy, left_turns, _))) =  scores.borrow().iter().find(|(w, _)| format!("{}", w) == "korea" ) {
                    <p>
                        { format!("{word}, {entropy}, {left_turns}")}
                    </p>
                }
            }
            <ul>
                { for scores.borrow().iter().take(10).map( |(word, (entropy, left_turns, _))| { html!{<li> { format!("{word}, {entropy}, {left_turns}") } </li>} } ) }
            </ul>
        </main>
    }
}
