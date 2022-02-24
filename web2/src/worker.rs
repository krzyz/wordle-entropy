use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use std::sync::Once;
use wasm_bindgen_futures::{spawn_local, JsFuture};
//use wasm_bindgen_rayon::init_thread_pool;
use wordle_entropy_core::solvers::solve_random;
use wordle_entropy_core::structs::WordN;

static INIT_THREAD_POOL: Once = Once::new();

pub struct WordleWorker {
    link: WorkerLink<Self>,
}

impl Worker for WordleWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = (Vec<WordN<5>>, usize);
    type Output = Vec<(f32, i32)>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        /*
        INIT_THREAD_POOL.call_once(|| {
            let window = web_sys::window().expect("Missing Window");
            let navigator = window.navigator();
            spawn_local(async move {
                log::info!("{}", navigator.hardware_concurrency() as usize);
                JsFuture::from(init_thread_pool(navigator.hardware_concurrency() as usize))
                    .await
                    .ok()
                    .unwrap();
            })
        });
        */

        let output = solve_random(&msg.0, msg.1);
        self.link.respond(id, output);
    }
}
