use crate::worker::WordleWorker;
use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridged, Worker};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::cmp::Ordering::Equal;
use std::rc::Rc;
use web_sys::{FocusEvent, HtmlCanvasElement, HtmlInputElement, Performance};
use wordle_entropy_core::data::parse_words;
use wordle_entropy_core::structs::WordN;
use yew::{
    classes, events::Event, function_component, html, use_effect_with_deps, use_mut_ref,
    use_node_ref, use_reducer, use_state, Callback, Html, Reducible, TargetCast, UseStateHandle,
};

fn draw_plot(canvas: HtmlCanvasElement, data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

    let mut data = data.iter().copied().collect::<Vec<_>>();
    data.sort_by(|v1, v2| v2.partial_cmp(v1).unwrap_or(Equal));

    root.fill(&WHITE)?;

    let y_max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
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

type WorkerOutput = <WordleWorker as Worker>::Output;
type Entropies = Rc<WorkerOutput>;

enum WordsAction {
    LoadWords(Vec<WordN<char, 5>>),
    StartCalc,
    EndCalc(<WordleWorker as Worker>::Output),
}

#[derive(PartialEq)]
struct WordsState {
    performance: Performance,
    perf_start: Option<f64>,
    perf_end: Option<f64>,
    running: bool,
    entropies: Entropies,
    words: Rc<Vec<WordN<char, 5>>>,
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
            entropies: WorkerOutput::new().into(),
            words: vec![].into(),
        }
    }
}

impl Reducible for WordsState {
    type Action = WordsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            WordsAction::LoadWords(words) => {
                let mut new_state = Self::default();
                new_state.words = Rc::new(words);
                new_state.into()
            }
            WordsAction::StartCalc => Self {
                performance: self.performance.clone(),
                perf_start: Some(self.performance.now()),
                perf_end: None,
                running: true,
                entropies: WorkerOutput::new().into(),
                words: self.words.clone(),
            }
            .into(),
            WordsAction::EndCalc(output) => Self {
                performance: self.performance.clone(),
                perf_start: self.perf_start,
                perf_end: Some(self.performance.now()),
                running: false,
                entropies: output.into(),
                words: self.words.clone(),
            }
            .into(),
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let word_state = use_reducer(WordsState::default);
    let file_input_node_ref = use_node_ref();
    let canvas_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);
    let selected_word = use_state(|| -> Option<WordN<char, 5>> { None });

    let cb = {
        let word_state = word_state.clone();
        let selected_word = selected_word.clone();
        move |output: <WordleWorker as Worker>::Output| {
            if let Some((word, _)) = output.iter().next() {
                selected_word.set(Some(word.clone()));
            }
            word_state.dispatch(WordsAction::EndCalc(output))
        }
    };

    {
        let canvas_node_ref = canvas_node_ref.clone();
        let word_state = word_state.clone();
        let selected_word = selected_word.clone();
        use_effect_with_deps(
            move |(word_state, selected_word)| {
                let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();

                let data = selected_word
                    .as_ref()
                    .map(|selected_word| {
                        let word_entropy = word_state
                            .entropies
                            .iter()
                            .find(|&(word, _)| word == selected_word);
                        word_entropy.map(|x| x.1 .2.values().map(|&x| x).collect::<Vec<_>>())
                    })
                    .flatten()
                    .unwrap_or(vec![]);

                draw_plot(canvas, &data[..]).unwrap();
                || ()
            },
            (word_state, selected_word),
        )
    }

    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));

    let onclick_run = {
        let word_state = word_state.clone();
        let worker = worker.clone();
        Callback::from(move |_| {
            worker.borrow_mut().send((*word_state.words).clone());
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

    let onload = {
        let word_state = word_state.clone();
        let file_reader = file_reader.clone();
        let file_input_node_ref = file_input_node_ref.clone();

        Callback::from(move |e: FocusEvent| {
            let word_state = word_state.clone();
            e.prevent_default();
            let file_input = file_input_node_ref.cast::<HtmlInputElement>().unwrap();
            let files = file_input
                .files()
                .map(|files| gloo_file::FileList::from(files));

            if let Some(files) = files {
                if let Some(file) = files.first() {
                    *file_reader.borrow_mut() = Some(read_as_text(&file, move |res| match res {
                        Ok(content) => {
                            word_state.dispatch(WordsAction::LoadWords(parse_words::<_, 5>(
                                content.lines(),
                            )));
                        }
                        Err(err) => {
                            log::info!("Reading file error: {err}");
                        }
                    }));
                }
            }
        })
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
        <main>
            <div class="container">
                <div class="columns">
                    <div class="column col-6 col-mx-auto">
                        <form onsubmit={onload}>
                            <input class="btn" ref={file_input_node_ref} type="file"/>
                            <button class="btn btn-primary">{"Load words"}</button>
                        </form>
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
                                word_state.entropies.iter().take(*max_words_shown).map(|(word, (entropy, left_turns, _))| {
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
                            }
                        </ul>
                    </div>
                </div>
            </div>
        </main>
    }
}
