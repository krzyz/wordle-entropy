use std::rc::Rc;

use serde_cbor::ser::to_vec_packed;
use yew::{
    function_component, html, use_effect_with_deps, use_mut_ref, use_reducer, Callback, Reducible,
};

use crate::components::select_words::{SelectWords, SelectedWords};
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::word_set::get_current_word_set;
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::Word;

enum SimulationStateAction {
    Initialize(Word, Vec<Word>),
}

#[derive(Clone, Default, PartialEq)]
struct SimulationState {
    current_word: Option<Word>,
    words_left: Vec<Word>,
}

impl Reducible for SimulationState {
    type Action = SimulationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SimulationStateAction::Initialize(word, words_left) => Rc::new(SimulationState {
                current_word: Some(word),
                words_left,
            }),
        }
    }
}

#[function_component(Simulation)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let selected_words = use_mut_ref(|| SelectedWords::Random(10));

    let simulation_state = use_reducer(|| SimulationState::default());
    let send_queue = use_mut_ref(|| -> Option<SimulationInput> { None });

    let on_words_set = {
        let selected_words = selected_words.clone();
        Callback::from(move |new_selected_words| {
            *selected_words.borrow_mut() = new_selected_words;
        })
    };

    let cb = {
        let send_queue = send_queue.clone();
        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(name) => log::info!("Set worker with: {name}"),
            WordleWorkerOutput::Simulation(output) => match output {
                SimulationOutput::StepComplete {
                    guess,
                    hints,
                    uncertainty,
                    scores,
                    answers,
                } => {
                    log::info!("{guess}, {hints}, {uncertainty}, {scores:#?}, {answers:#?}");
                    if answers.len() > 1 {
                        let next_guess = scores.iter().next().unwrap().0.word.clone();
                        *send_queue.borrow_mut() =
                            Some(SimulationInput::Continue(Some(next_guess)));
                    }
                }
                SimulationOutput::Stopped => todo!(),
            },
            WordleWorkerOutput::Err(error) => {
                log::info!("{error}");
            }
            _ => log::info!("Unexpected worker output"),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    if let Some(input) = send_queue.borrow().clone() {
        let worker = worker.clone();
        worker.send(WordleWorkerInput::Simulation(input));
    }

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                log::info!("send set work set");
                worker.send(WordleWorkerInput::SetWordSetEncoded(
                    to_vec_packed(&word_set.reduce_entropies(10)).unwrap(),
                ));
                log::info!("finished send");
                || ()
            },
            word_set_name,
        )
    }

    let on_run_button_click = {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let selected_words = selected_words.clone();
        let simulation_state = simulation_state.clone();

        Callback::from(move |_| {
            let mut words = match *selected_words.borrow() {
                SelectedWords::Random(n) => {
                    word_set.dictionary.words.iter().take(n).cloned().collect()
                }
                SelectedWords::Custom(ref words) => words.clone(),
            };

            if words.len() > 0 {
                let word = words.swap_remove(0);
                simulation_state.dispatch(SimulationStateAction::Initialize(word.clone(), words));
                worker.send(WordleWorkerInput::Simulation(SimulationInput::Start(
                    word, None,
                )));
            } else {
                log::info!("..no words?");
            }
        })
    };

    html! {
        <section>
            <SelectWords dictionary={word_set.dictionary.clone()} {on_words_set} />
            <button class="btn btn-primary" onclick={on_run_button_click}>{ "Run" }</button>
        </section>
    }
}
