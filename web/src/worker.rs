use gloo_worker::{HandlerId, Private, Worker, WorkerLink};
use wordle_entropy_core::entropy::calculate_entropies;
use wordle_entropy_core::solvers::expected_turns;
use wordle_entropy_core::structs::{WordN, GuessHints};

pub struct WordleWorker {
    link: WorkerLink<Self>,
}

impl Worker for WordleWorker {
    type Reach = Private<Self>;
    type Message = ();
    type Input = (Vec<WordN<char, 5>>, Vec<WordN<char, 5>>);
    type Output = Vec<(WordN<char, 5>, (f32, f32, GuessHints<5>))>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

   fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let (words, answers) = &msg;
        let entropies = calculate_entropies(&words, answers);
        let uncertainty = (answers.len() as f32).log2();

        let scores = entropies
            .into_iter()
            .map(|(g, (entropy, guess_hints))| {
                let prob = if answers.contains(&g) {
                    1. / (answers.len() as f32)
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(uncertainty - entropy, 0., 1.6369421, -0.029045254)
                    * (1. - prob);

                (g, (entropy, left_diff, guess_hints))
            })
            .collect::<Vec<_>>();

        self.link.respond(id, scores);
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}
