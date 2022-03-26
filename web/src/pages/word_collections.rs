use yew::{function_component, html};

#[function_component(WordCollections)]
pub fn view() -> Html {
    html! {
        <container>
            <h1>
                { "Word collections" }
            </h1>
        </container>
    }
}
