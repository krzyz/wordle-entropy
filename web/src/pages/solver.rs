use std::{collections::VecDeque, rc::Rc};

use bounce::use_atom_setter;
use serde_cbor::ser::to_vec_packed;
use yew::{function_component, html, use_effect_with_deps, use_state};

use crate::components::{SimulationDetail, ToastOption, ToastType};
use crate::word_set::get_current_word_set;
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;

use super::GuessStep;

#[function_component(Solver)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let history = use_state(|| -> VecDeque<(usize, Vec<GuessStep>)> { VecDeque::new() });
    let set_toast = use_atom_setter::<ToastOption>();

    let cb = {
        let set_toast = set_toast.clone();

        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(_name) => (),
            _ => set_toast(ToastOption::new(
                "Unexpected worker output".to_string(),
                ToastType::Error,
            )),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                worker.send(WordleWorkerInput::SetWordSetEncoded(
                    to_vec_packed(&word_set.reduce_entropies(10)).unwrap(),
                ));
                || ()
            },
            word_set_name,
        )
    }

    html! {
        <section>
            <SimulationDetail history={(*history).clone()} word_set={word_set.clone()} />
        </section>
    }
}
