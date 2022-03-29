use bounce::use_atom;
use yew::{function_component, html, Properties};

use crate::main_app::WordSetSelection;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
}

#[function_component(Solver)]
pub fn view(props: &Props) -> Html {
    let selected = use_atom::<WordSetSelection>();
    if props.name != "" {
        selected.set(WordSetSelection(Some(props.name.clone())))
    }

    html! {
        <section>
            <div>
                <div>
                    <h1>
                        { "Page not found" }
                    </h1>
                    <h2>
                        { "Page page does not seem to exist" }
                    </h2>
                </div>
            </div>
        </section>
    }
}
