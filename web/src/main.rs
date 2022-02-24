mod app;

use app::App;
pub use wasm_bindgen_rayon::init_thread_pool;
use wasm_bindgen_futures::{JsFuture, spawn_local};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let window = web_sys::window().expect("Missing Window");
    let navigator = window.navigator();
    spawn_local(async move {
        log::info!("{}", navigator.hardware_concurrency() as usize);
        //JsFuture::from(init_thread_pool(navigator.hardware_concurrency() as usize)).await.ok().unwrap();
    });
    yew::start_app::<App>();
}
