use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use std::cmp::Ordering::Equal;
use wordle_entropy_core::indexmap::IndexMap;
use wordle_entropy_core::entropy::calculate_entropies;
use wordle_entropy_core::solvers::expected_turns;
use wordle_entropy_core::structs::WordN;

pub struct WordleWorker {
    link: WorkerLink<Self>,
}

impl Worker for WordleWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = Vec<WordN<char, 5>>;
    type Output = Vec<(WordN<char, 5>, f32, f32)>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let words = &msg;
        let answers = &msg;
        let entropies = calculate_entropies(&words, answers);
        let uncertainty = (words.len() as f32).log2();

        let mut scores = entropies
            .into_iter()
            .map(|(g, entropy)| {
                let prob = if answers.contains(&g) {
                    1. / (answers.len() as f32)
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(uncertainty - entropy, 0., 1.6369421, -0.029045254)
                    * (1. - prob);

                (g, entropy, left_diff)
            })
            .collect::<Vec<_>>();

        scores.sort_by(|&(_, _, score1), &(_, _, score2)| {
            score1.partial_cmp(&score2).unwrap_or(Equal)
        });

        let best_scores: Vec<_> = scores
            .iter()
            .take(10)
            .map(|(word, entropy, score)| (word.clone(), *entropy, *score))
            .collect();

        self.link.respond(id, best_scores);
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}
