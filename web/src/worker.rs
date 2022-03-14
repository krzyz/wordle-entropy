use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use wordle_entropy_core::solvers::solve_random;
use wordle_entropy_core::structs::WordN;

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
        let output = solve_random(&msg.0, msg.1);
        self.link.respond(id, output);
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}