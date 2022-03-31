use crate::main_app::get_current_word_set;
use crate::word_set::{WordSetVec, WordSetVecAction};
use crate::worker::WordleWorker;
use bounce::use_slice_dispatch;
use gloo_worker::{Bridged, Worker};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::cmp::Ordering::Equal;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, HtmlInputElement, Performance};
use wordle_entropy_core::structs::WordN;
use yew::{
    classes, events::Event, function_component, html, use_effect_with_deps, use_mut_ref,
    use_node_ref, use_reducer, use_state, Callback, Html, Reducible, TargetCast, UseStateHandle,
};

fn draw_plot(canvas: HtmlCanvasElement, data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

    let mut data = data.iter().copied().collect::<Vec<_>>();
    data.sort_by(|v1, v2| v2.partial_cmp(v1).unwrap_or(Equal));

    root.fill(&WHITE)?;

    let y_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_max = if data.len() > 0 { y_max + 0.02 } else { 1.0 };

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .margin(8u32)
        .build_cartesian_2d((0..data.len()).into_segmented(), 0.0..y_max)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .disable_x_axis()
        .y_desc("probability")
        .axis_desc_style(("sans-serif", 15u32))
        .draw()?;

    chart.draw_series(data.into_iter().enumerate().map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        Rectangle::new([(x0, 0.), (x1, y)], BLUE.filled())
    }))?;

    root.present().expect("Unable to draw");

    Ok(())
}

enum WordsAction {
    StartCalc,
    EndCalc,
}

#[derive(PartialEq)]
struct WordsState {
    performance: Performance,
    perf_start: Option<f64>,
    perf_end: Option<f64>,
    running: bool,
}

impl Default for WordsState {
    fn default() -> Self {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        Self {
            performance,
            perf_start: None,
            perf_end: None,
            running: false,
        }
    }
}

impl Reducible for WordsState {
    type Action = WordsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            WordsAction::StartCalc => Self {
                performance: self.performance.clone(),
                perf_start: Some(self.performance.now()),
                perf_end: None,
                running: true,
            }
            .into(),
            WordsAction::EndCalc => Self {
                performance: self.performance.clone(),
                perf_start: self.perf_start,
                perf_end: Some(self.performance.now()),
                running: false,
            }
            .into(),
        }
    }
}

#[function_component(EntropyCalculation)]
pub fn app() -> Html {
    let word_set = get_current_word_set();
    let dispatch_word_sets = use_slice_dispatch::<WordSetVec>();

    let word_state = use_reducer(WordsState::default);
    let canvas_node_ref = use_node_ref();
    let selected_word = {
        let word_set = word_set.clone();
        use_state(|| {
            if let Some(entropies) = word_set.entropies {
                entropies.iter().next().map(|(entropies_data, _)| entropies_data.word.clone())
            } else {
                None
            }
        })
    };

    let cb = {
        let word_state = word_state.clone();
        let selected_word = selected_word.clone();
        let dispatch_word_sets = dispatch_word_sets.clone();
        move |(name, entropies_output): <WordleWorker as Worker>::Output| {
            if let Some((entropies_data, _)) = entropies_output.iter().next() {
                selected_word.set(Some(entropies_data.word.clone()));
            }

            log::info!("Setting entropy: {}", name);
            dispatch_word_sets(WordSetVecAction::SetEntropy(name, entropies_output));
            word_state.dispatch(WordsAction::EndCalc);
        }
    };

    {
        let canvas_node_ref = canvas_node_ref.clone();
        let selected_word = selected_word.clone();
        let word_set = word_set.clone();
        use_effect_with_deps(
            move |selected_word| {
                let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
                let word_set = word_set.clone();

                let data = selected_word
                    .as_ref()
                    .map(|selected_word| {
                        let word_entropy = if let Some(entropies) = word_set.entropies.clone() {
                            entropies
                                .iter()
                                .find(|&(entropies_data, _)| &entropies_data.word == selected_word)
                                .cloned()
                        } else {
                            None
                        };
                        word_entropy.map(|(entropies_data, _)| {
                            entropies_data
                                .probabilities
                                .into_values()
                                .collect::<Vec<_>>()
                        })
                    })
                    .flatten()
                    .unwrap_or(vec![]);

                draw_plot(canvas, &data[..]).unwrap();
                || ()
            },
            selected_word,
        )
    }

    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));

    let onclick_run = {
        let word_state = word_state.clone();
        let worker = worker.clone();
        let word_set = word_set.clone();
        Callback::from(move |_| {
            log::info!("run");
            log::info!("found dictionary of: {}", word_set.name);
            worker.borrow_mut().send((word_set.name.clone(), word_set.dictionary.clone()));
            log::info!("dictionary send");
            word_state.dispatch(WordsAction::StartCalc);
        })
    };

    let onclick_word = {
        |word: WordN<_, 5>, selected_word: UseStateHandle<Option<WordN<_, 5>>>| {
            Callback::from(move |_| {
                selected_word.set(Some(word.clone()));
            })
        }
    };

    let max_words_shown = use_state(|| 10);
    let on_max_words_shown_change = {
        let max_words_shown = max_words_shown.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(new_num) = input.value().parse::<usize>().ok() {
                max_words_shown.set(new_num);
            }
        })
    };

    html! {
        <div class="container">
            <div class="columns">
                <div class="column col-6 col-mx-auto">
                    <button class="btn btn-primary" disabled={word_state.running} onclick={onclick_run}>{"Run"}</button>
                    {
                        if word_state.running {
                            html!(<div class="d-inline-block loading p-2"></div>)
                        } else {
                            html!()
                        }
                    }
                </div>
            </div>
            <div class="columns">
                <div class="column">
                    <canvas ref={canvas_node_ref} id="canvas" width="800" height="400"></canvas>
                    if let (Some(perf_start), Some(perf_end)) = (word_state.perf_start, word_state.perf_end) {
                        <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
                    }
                </div>
                <div class="column">
                    <label for="max_words_shown_input">{"Max words shown:"}</label>
                    <input id="max_words_shown_input" onchange={on_max_words_shown_change} value={(*max_words_shown).to_string()}/>
                    <ul class="words_entropies_list">
                        {
                            if let Some(ref entropies) = word_set.entropies {
                                entropies
                                    .iter().take(*max_words_shown).map(|(entropy_data, left_turns)| {
                                        let word = &entropy_data.word;
                                        let entropy = &entropy_data.entropy;
                                        html! {
                                            <li
                                                key={format!("{word}")}
                                                class={classes!(
                                                    "c-hand",
                                                    (*selected_word).clone().map(|selected_word| { *word == selected_word }).map(|is_selected| is_selected.then(|| Some("text-primary")))
                                                )}
                                                onclick={onclick_word(word.clone(), selected_word.clone())}
                                            >
                                                {format!("{word}: {entropy}, {left_turns}")}
                                            </li>
                                        }
                                    }).collect::<Html>()
                            } else {
                                html! {<> </>}
                            }
                        }
                    </ul>
                </div>
            </div>
        </div>
    }
}
