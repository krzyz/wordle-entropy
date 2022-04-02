use gloo_worker::{HandlerId, Public, Worker, WorkerLink};
use std::cmp::Ordering::Equal;
use wordle_entropy_core::entropy::calculate_entropies;
use wordle_entropy_core::solvers::expected_turns;
use wordle_entropy_core::structs::{Dictionary, EntropiesData};

pub struct WordleWorker {
    link: WorkerLink<Self>,
}

impl Worker for WordleWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = (String, Dictionary<5>);
    type Output = (String, Vec<(EntropiesData<5>, f64)>);

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, (name, dictionary): Self::Input, id: HandlerId) {
        let answers = (0..dictionary.words.len()).collect::<Vec<_>>();
        let entropies = calculate_entropies(&dictionary, &answers);

        let uncertainty = (dictionary.words.len() as f64).log2();
        let prob_norm: f64 = answers.iter().map(|&i| dictionary.probabilities[i]).sum();

        let mut scores = entropies
            .into_iter()
            .enumerate()
            .map(|(i, entropies_data)| {
                let prob = if answers.contains(&i) {
                    dictionary.probabilities[i] / prob_norm
                } else {
                    0.
                };

                // the less the better
                let left_diff = expected_turns(
                    uncertainty - entropies_data.entropy,
                    0.,
                    1.6369421,
                    -0.029045254,
                ) * (1. - prob);

                (entropies_data, left_diff)
            })
            .collect::<Vec<_>>();

        scores.sort_by(|&(_, score1), &(_, score2)| score1.partial_cmp(&score2).unwrap_or(Equal));

        self.link.respond(id, (name, scores));
    }

    fn name_of_resource() -> &'static str {
        "wordle_entropy_web.js"
    }

    fn is_module() -> bool {
        true
    }
}
