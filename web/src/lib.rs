
#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

pub mod app;
mod worker;

use gloo_worker::PublicWorker;
use wasm_bindgen::prelude::*;
pub use wasm_bindgen_rayon::init_thread_pool;
//use yew_agent::Threaded;

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};

    // check if we are the main/UI thread
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        wasm_logger::init(wasm_logger::Config::default());
        yew::start_app::<app::App>();
    } else {
        <worker::WordleWorker as PublicWorker>::register();
    }
}