use crate::worker::WordleWorker;
use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridged, Worker};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{FocusEvent, HtmlCanvasElement, HtmlInputElement, Performance};
use wordle_entropy_core::data::parse_words;
use wordle_entropy_core::structs::WordN;
use yew::{
    function_component, html, use_effect_with_deps, use_mut_ref, use_node_ref, use_reducer,
    Callback, Reducible,
};

fn draw_plot(canvas: HtmlCanvasElement, data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .margin(5u32)
        .caption("Histogram Test", ("sans-serif", 50.0f32))
        .build_cartesian_2d(
            (0..data.len()).into_segmented(),
            0.0..(data.iter().copied().fold(f32::NEG_INFINITY, f32::max)),
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Bucket")
        .axis_desc_style(("sans-serif", 15u32))
        .draw()?;

    chart.draw_series(data.into_iter().enumerate().map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0.), (x1, *y)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))?;

    root.present().expect("Unable to draw");

    Ok(())
}

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
    entropies: Rc<RefCell<<WordleWorker as Worker>::Output>>,
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
            entropies: Rc::new(RefCell::new(<WordleWorker as Worker>::Output::new().into())),
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
                entropies: self.entropies.clone(),
                words: self.words.clone(),
            }
            .into(),
            WordsAction::EndCalc(output) => {
                *self.entropies.borrow_mut() = output;
                Self {
                    performance: self.performance.clone(),
                    perf_start: self.perf_start,
                    perf_end: Some(self.performance.now()),
                    entropies: self.entropies.clone(),
                    words: self.words.clone(),
                }
                .into()
            }
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let word_state = use_reducer(WordsState::default);
    let file_input_node_ref = use_node_ref();
    let canvas_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let cb = {
        let word_state = word_state.clone();
        move |output: <WordleWorker as Worker>::Output| {
            word_state.dispatch(WordsAction::EndCalc(output))
        }
    };

    {
        let canvas_node_ref = canvas_node_ref.clone();
        let word_state = word_state.clone();
        use_effect_with_deps(
            move |word_state| {
                log::info!("called");
                if let Some(entropies) = word_state.entropies.borrow().iter().next() {
                    let data = entropies.1 .2.values().map(|&x| x).collect::<Vec<_>>();
                    let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
                    log::info!("{data:#?}");
                    draw_plot(canvas, &data[..]).unwrap();
                }
                || ()
            },
            word_state,
        )
    }

    let worker = use_mut_ref(|| WordleWorker::bridge(Rc::new(cb)));

    let onclick = {
        let word_state = word_state.clone();
        let worker = worker.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            worker.borrow_mut().send((*word_state.words).clone());
            word_state.dispatch(WordsAction::StartCalc);
        })
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

            log::info!("{files:#?}");

            if let Some(files) = files {
                log::info!("Some files");
                if let Some(file) = files.first() {
                    log::info!("File first");
                    *file_reader.borrow_mut() = Some(read_as_text(&file, move |res| {
                        log::info!("Reading ");
                        match res {
                            Ok(content) => {
                                log::info!("Read file");
                                word_state.dispatch(WordsAction::LoadWords(parse_words::<_, 5>(
                                    content.lines(),
                                )));
                            }
                            Err(err) => {
                                log::info!("Reading file error: {err}");
                            }
                        }
                    }));
                }
            }
        })
    };

    html! {
        <main>
            <form onsubmit={onload}>
                <input ref={file_input_node_ref} type="file"/>
                <button>{"Load words"}</button>
            </form>
            <button {onclick}>{"Run"}</button>
            <br />
            <canvas ref={canvas_node_ref} id="canvas" width="600" height="400"></canvas>
            if let (Some(perf_start), Some(perf_end)) = (word_state.perf_start, word_state.perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
        </main>
    }
}
