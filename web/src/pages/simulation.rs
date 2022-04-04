use std::rc::Rc;

use yew::{function_component, html, use_mut_ref, Callback};

use crate::components::select_words::{SelectWords, SelectedWords};
use crate::word_set::get_current_word_set;
use crate::worker::WordleWorkerOutput;
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

    let on_run_button_click = {
        let selected_words = selected_words.clone();
        Callback::from(move |_| {
            log::info!("{:#?}", *selected_words.borrow());
        })
    };

    let cb = {
        move |output: WordleWorkerOutput| match output {
            _ => log::info!("Unexpected worker output"),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    html! {
        <section>
            <SelectWords dictionary={word_set.dictionary} {on_words_set} />
            <button class="btn btn-primary" onclick={on_run_button_click}>{ "Run" }</button>
        </section>
    }
}
