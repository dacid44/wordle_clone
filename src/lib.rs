#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod draw;
mod utils;

pub use app::WordleApp;
pub use utils::Args;

include!(concat!(env!("OUT_DIR"), "/word_list.rs"));

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let search_params = web_sys::UrlSearchParams::new_with_str(
        &web_sys::window().expect("no global window").location().search()?,
    )?;

    let word = if let Some(arg) = search_params.get("word") {
        arg.to_uppercase()
    } else if let Some(arg) = search_params.get("bword").map(|x| utils::decode(x).ok()).flatten() {
        arg
    } else {
        String::new()
    };

    let app = if WORD_LIST.contains(&&*word) {
        WordleApp::with_args(utils::Args { word: Some(word) })
    } else {
        WordleApp::default()
    };

    eframe::start_web(canvas_id, Box::new(app))
}
