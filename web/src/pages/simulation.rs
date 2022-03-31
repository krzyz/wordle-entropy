use yew::{function_component, html};

#[function_component(Simulation)]
pub fn view() -> Html {
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
