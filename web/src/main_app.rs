use crate::components::word_set_select::WordSetSelect;
use crate::pages::{
    entropy_calculation::EntropyCalculation, page_not_found::PageNotFound, simulation::Simulation,
    solver::Solver, word_sets::WordSets,
};
use bounce::BounceRoot;
use yew::{function_component, html, Html};
use yew_router::components::Link;
use yew_router::{BrowserRouter, Routable, Switch};

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/collections")]
    WordSets,
    #[at("/entropy")]
    EntropyCalculation,
    #[at("/simulation")]
    Simulation,
    #[at("/solver")]
    Solver,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(MainApp)]
pub fn view() -> Html {
    html! {
        <BounceRoot>
            <BrowserRouter>
                <nav class="navbar">
                    <section class="navbar-section">
                        <Link<Route> classes="btn btn-link" to={Route::WordSets}>
                            { "Word sets" }
                        </Link<Route>>
                        <Link<Route> classes="btn btn-link" to={Route::EntropyCalculation}>
                            { "Entropy Calculation" }
                        </Link<Route>>
                        <Link<Route> classes="btn btn-link" to={Route::Simulation}>
                            { "Simulation" }
                        </Link<Route>>
                        <Link<Route> classes="btn btn-link" to={Route::Solver}>
                            { "Solver" }
                        </Link<Route>>
                    </section>
                    <section>
                        <WordSetSelect />
                    </section>
                </nav>
                <main>
                    <Switch<Route> render={Switch::render(switch)} />
                </main>
            </BrowserRouter>
        </BounceRoot>
    }
}



fn switch(routes: &Route) -> Html {
    match routes.clone() {
        Route::Home | Route::WordSets => {
            html! { <WordSets /> }
        }
        Route::EntropyCalculation => {
            html! { <EntropyCalculation /> }
        }
        Route::Simulation {} => {
            html! { <Simulation /> }
        }
        Route::Solver {} => {
            html! { <Solver /> }
        }
        Route::NotFound => {
            html! { <PageNotFound /> }
        }
    }
}
