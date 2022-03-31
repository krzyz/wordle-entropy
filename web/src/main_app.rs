use crate::pages::{
    entropy_calculation::EntropyCalculation, page_not_found::PageNotFound, simulation::Simulation,
    solver::Solver, word_sets::WordSets,
};
use crate::word_set::{WordSet, WordSetVec, WordSetVecAction};
use bounce::{use_atom, use_slice, Atom, BounceRoot};
use reqwest::StatusCode;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use wordle_entropy_core::data::parse_words;
use wordle_entropy_core::structs::Dictionary;
use yew::events::Event;
use yew::{function_component, html, use_effect_with_deps, Callback, Html, TargetCast};
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

#[derive(Default, PartialEq, Atom)]
pub struct WordSetSelection(pub Option<String>);

#[function_component(WordSetSelect)]
pub fn word_set_select() -> Html {
    let word_sets = use_slice::<WordSetVec>();
    let selected = use_atom::<WordSetSelection>();

    {
        let word_sets = word_sets.clone();
        use_effect_with_deps(
            move |_| {
                if word_sets.0.len() == 0 {
                    spawn_local(async move {
                        let client = reqwest::Client::new();
                        let response = client
                                .get("https://wordle.realcomplexity.com/data/words-scrabble-with_probs.csv")
                                .send()
                                .await.unwrap();

                        match response.status() {
                            StatusCode::OK => {
                                let text = response.text().await.unwrap();
                                let dictionary = parse_words::<_, 5>(text.lines());
                                word_sets.dispatch(WordSetVecAction::Set(
                                    (*word_sets).extend_with(WordSet::from_dictionary(
                                        "Polish words scrabble".to_string(),
                                        dictionary,
                                    )),
                                ));
                                log::info!("Loaded from url");
                            }
                            _ => log::info!("Error loading csv"),
                        }
                    });
                } else {
                }
                || ()
            },
            (),
        );
    }

    let onchange = {
        let selected = use_atom::<WordSetSelection>();
        Callback::from(move |e: Event| {
            log::info!("on select change");
            let select: HtmlInputElement = e.target_unchecked_into();
            selected.set(WordSetSelection(Some(select.value().clone())));
        })
    };

    if word_sets.0.len() > 0 && *selected == WordSetSelection(None) {
        selected.set(WordSetSelection(Some(
            word_sets.0.iter().next().unwrap().name.clone(),
        )));
    }

    html! {
        <select name="word_sets" {onchange}>
            {
                word_sets.0.iter().map(|word_set| {
                    let name = word_set.name.clone();
                    let name_optional = Some(name.clone());
                    html! {
                        <option value={name.clone()} selected={selected.0 == name_optional }> {name} </option>
                    }
                }).collect::<Html>()
            }
        </select>

    }
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
                <MainSection />
            </BrowserRouter>
        </BounceRoot>
    }
}

#[function_component(MainSection)]
pub fn main_section() -> Html {
    let word_sets = use_slice::<WordSetVec>();
    let select = use_atom::<WordSetSelection>();
    let word_set = word_sets
        .0
        .iter()
        .find(|word_set| Some(&word_set.name) == select.0.as_ref())
        .cloned()
        .unwrap_or(WordSet::from_dictionary(
            "invalid_word_set".to_string(),
            Dictionary::new(vec![], vec![]),
        ));

    log::info!("Word set: {}", word_set.name);

    html! {
        <main>
            <Switch<Route> render={Switch::render(move |r| switch(r, word_set.clone()))} />
        </main>
    }
}

fn switch(routes: &Route, word_set: WordSet) -> Html {
    match routes.clone() {
        Route::Home | Route::WordSets => {
            html! { <WordSets /> }
        }
        Route::EntropyCalculation => {
            html! { <EntropyCalculation {word_set} /> }
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
