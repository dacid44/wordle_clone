#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use std::env;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = env::args().collect();
    let app = if args.len() > 1 && wordle_clone::WORD_LIST.contains(&&*args[1].to_uppercase()) {
        wordle_clone::WordleApp::with_args(wordle_clone::Args {
            word: Some(args[1].to_uppercase()),
        })
    } else {
        wordle_clone::WordleApp::default()
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
