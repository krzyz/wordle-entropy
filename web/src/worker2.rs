use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use std::cmp::Ordering::Equal;
use wordle_entropy_core::entropy::calculate_entropies;
use wordle_entropy_core::solvers::expected_turns;
use wordle_entropy_core::structs::WordN;

use crate::demo::generate;

pub struct DemoWorker {
    link: WorkerLink<Self>,
}

impl Worker for DemoWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = Vec<WordN<char, 5>>;
    //type Output = Vec<(WordN<char, 5>, f32, f32)>;
    type Output = f32;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let words = &msg;
        let answers = &msg;
        let entropy = calculate_entropies(words, answers).into_iter().next().unwrap().1.0;

        self.link.respond(id, entropy);
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}
