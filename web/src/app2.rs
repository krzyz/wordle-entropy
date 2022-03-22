use std::rc::Rc;
use gloo_file::callbacks::read_as_text;
use gloo_worker::{Bridged, Worker};
use web_sys::{FocusEvent, HtmlInputElement, HtmlCanvasElement};
use wordle_entropy_core::data::parse_words;
use yew::{
    events::Event, function_component, html, use_mut_ref, use_node_ref, use_state, Callback,
    TargetCast,
};
use crate::worker2::DemoWorker;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

fn draw_plot(canvas: HtmlCanvasElement) -> Result<(), Box<dyn std::error::Error>> {
    let root = CanvasBackend::with_canvas_object(canvas).unwrap().into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35u32)
        .y_label_area_size(40u32)
        .margin(5u32)
        .caption("Histogram Test", ("sans-serif", 50.0f32))
        .build_cartesian_2d((0u32..10u32).into_segmented(), 0u32..10u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Bucket")
        .axis_desc_style(("sans-serif", 15u32))
        .draw()?;

    let data = [
        0u32, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
    ];

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.iter().map(|x: &u32| (*x, 1))),
    )?;

    root.present().expect("Unable to draw");

    Ok(())
}

#[function_component(App2)]
pub fn app() -> Html {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = use_state(|| {
        window
            .performance()
            .expect("performance should be available")
    });
    let perf_start = use_state(|| -> Option<f64> { None });
    let perf_end = use_state(|| -> Option<f64> { None });
    let file_input_node_ref = use_node_ref();
    let canvas_node_ref = use_node_ref();
    let file_reader = use_mut_ref(|| None);

    let cb = {
        let performance = performance.clone();
        let perf_end = perf_end.clone();
        let canvas_node_ref = canvas_node_ref.clone();
        move |word: <DemoWorker as Worker>::Output| {
            perf_end.set(Some(performance.now()));
            log::info!("{word}");
            let canvas = canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
            draw_plot(canvas).unwrap();
        }
    };


    let words = use_state(|| vec![]);
    let worker = use_mut_ref(|| DemoWorker::bridge(Rc::new(cb)));

    let onclick = {
        let words = words.clone();
        let worker = worker.clone();
        let performance = performance.clone();
        let perf_start = perf_start.clone();
        let perf_end = perf_end.clone();
        Callback::from(move |_| {
            log::info!("call worker");
            perf_start.set(Some(performance.now()));
            perf_end.set(None);
            worker.borrow_mut().send((*words).clone());
        })
    };

    let onload = {
        let words = words.clone();
        let file_reader = file_reader.clone();
        let file_input_node_ref = file_input_node_ref.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let words = words.clone();
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
                                words.set(parse_words::<_, 5>(content.lines()));
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
            if let (Some(perf_start), Some(perf_end)) = (*perf_start, *perf_end) {
                <p> { format!("{:.3} ms", perf_end - perf_start) } </p>
            }
        </main>
    }
}