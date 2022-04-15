#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

mod components;
pub mod main_app;
mod pages;
mod plots;
mod simulation;
mod word_set;
mod worker;
mod worker_atom;

use gloo_worker::PublicWorker;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
pub use wasm_bindgen_rayon::init_thread_pool;

const WORD_SIZE: usize = 5;
pub type Word = wordle_entropy_core::structs::WordN<char, WORD_SIZE>;
pub type Hints = wordle_entropy_core::structs::HintsN<WORD_SIZE>;
pub type Dictionary = wordle_entropy_core::structs::Dictionary<WORD_SIZE>;
pub type EntropiesData = wordle_entropy_core::structs::EntropiesData<WORD_SIZE>;
pub type Knowledge = wordle_entropy_core::structs::knowledge::KnowledgeN<WORD_SIZE>;

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};

    // check if we are the main/UI thread
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        wasm_logger::init(wasm_logger::Config::default());
        yew::start_app::<main_app::MainApp>();
    } else {
        spawn_local(async move {
            match JsFuture::from(init_thread_pool(12)).await {
                Ok(_) => (),
                Err(e) => log::error!("error: {:#?}", e),
            }
            <worker::WordleWorker as PublicWorker>::register();
        });
    }
}
