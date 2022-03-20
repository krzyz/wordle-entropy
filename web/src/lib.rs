
#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

pub mod app;
pub mod worker;

use gloo_worker::PrivateWorker;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};

    // check if we are the main/UI thread
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        wasm_logger::init(wasm_logger::Config::default());
        yew::start_app::<app::App>();
    } else {
        <worker::WordleWorker as PrivateWorker>::register();
    }
}