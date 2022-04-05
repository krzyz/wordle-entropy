use std::rc::Rc;

use serde_cbor::ser::to_vec_packed;
use yew::{function_component, html, use_mut_ref, Callback};

use crate::components::select_words::{SelectWords, SelectedWords};
use crate::word_set::get_current_word_set;
use crate::worker::{WordleWorkerOutput, WordleWorkerInput};
use crate::worker_atom::WordleWorkerAtom;

#[function_component(Simulation)]
pub fn view() -> Html {
    let word_set = get_current_word_set();
    let selected_words = use_mut_ref(|| SelectedWords::Random(10));

    let on_words_set = {
        let selected_words = selected_words.clone();
        Callback::from(move |new_selected_words| {
            *selected_words.borrow_mut() = new_selected_words;
        })
    };


    let cb = {
        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(name) => log::info!("Set worker with: {name}"),
            WordleWorkerOutput::Err(error) => {
                log::info!("{error}");
            }
            _ => log::info!("Unexpected worker output"),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    let on_run_button_click = {
        let worker = worker.clone();
        let word_set = word_set.clone();

        Callback::from(move |_| {
            log::info!("send set work set");
            worker.send(WordleWorkerInput::SetWordSetEncoded(
                to_vec_packed(&word_set).unwrap(),
            ));
            log::info!("finished send");
        })
    };

    html! {
        <section>
            <SelectWords dictionary={word_set.dictionary} {on_words_set} />
            <button class="btn btn-primary" onclick={on_run_button_click}>{ "Run" }</button>
        </section>
    }
}
