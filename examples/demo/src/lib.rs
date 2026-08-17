//! mogwai-md demo entry point.
//!
//! Renders a representative markdown document into the page body. Run with
//! `trunk serve` from the repo root.

use mogwai::{prelude::*, web::Web};
use mogwai_md::Markdown;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).ok();

    let src = include_str!("../sample.md");
    let md = Markdown::<Web>::new(src);
    mogwai::web::body().append_child(&md);
}
