use crate::pages::{
    entropy_calculation::EntropyCalculation, page_not_found::PageNotFound, simulation::Simulation,
    solver::Solver, word_sets::WordSets,
};
use crate::word_set::{WordSetVec, WordSet};
use std::cell::RefCell;
use std::rc::Rc;
use yew::{function_component, html, use_state, ContextProvider, Html};
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
    #[at("/solver/:id")]
    Solver,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn get_initial_word_sets() -> WordSetVec {
    vec![Rc::new(RefCell::new(WordSet::new(0, "Default Polish".to_string())))]
}

#[function_component(MainApp)]
pub fn view() -> Html {
    let word_sets_ctx = use_state(get_initial_word_sets);

    html! {
        <ContextProvider<WordSetVec> context={(*word_sets_ctx).clone()}>
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
                        <select name="word_sets">
                            {
                                word_sets_ctx.iter().map(|word_set| {
                                    let name = word_set.borrow().name.clone();
                                    html! {
                                        <option value={name.clone()}> {name} </option>
                                    }
                                }).collect::<Html>()
                            }
                        </select>
                    </section>
                </nav>
                <main>
                    <Switch<Route> render={Switch::render(switch)} />
                </main>
            </BrowserRouter>
        </ContextProvider<WordSetVec>>
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
        Route::Simulation => {
            html! { <Simulation /> }
        }
        Route::Solver => {
            html! { <Solver /> }
        }
        Route::NotFound => {
            html! { <PageNotFound /> }
        }
    }
}
