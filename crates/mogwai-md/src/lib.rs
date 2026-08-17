//! # mogwai-md
//!
//! A markdown rendering widget for [mogwai](https://github.com/schell/mogwai).
//!
//! `mogwai-md` parses markdown using [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
//! and lowers the event stream directly into a mogwai view tree. Because it
//! builds real DOM elements (not an HTML string), the same widget works in WASM
//! (`mogwai::web::Web`) and server-side rendering (`mogwai::ssr::Ssr`).
//!
//! ## Quickstart
//!
//! ```no_run
//! use mogwai::prelude::*;
//! use mogwai::web::Web;
//! use mogwai_md::Markdown;
//!
//! let md = Markdown::<Web>::new("# Hello, world\n\nThis is **mogwai-md**.");
//! mogwai::web::body().append_child(&md);
//! ```
//!
//! See the `examples/demo` crate for a trunk-servable example.
#![forbid(unsafe_code)]

pub use pulldown_cmark;

mod builder;
mod inline;
mod slug;

pub use builder::{append_children, children, default_options};

use mogwai::prelude::*;

/// A read-only markdown rendering widget.
///
/// Generic over `V: View` so the same widget works in WASM
/// (`mogwai::web::Web`) and SSR (`mogwai::ssr::Ssr`).
///
/// Construct with [`Markdown::new`] and update in place with
/// [`Markdown::set_source`]. The widget is a `<div class="mogwai-md">`
/// containing the rendered markdown as a tree of child elements.
#[derive(ViewChild)]
pub struct Markdown<V: View> {
    #[child]
    root: V::Element,
    /// Top-level block elements appended to `root`. Tracked so
    /// [`Markdown::set_source`] can remove them by identity before rebuilding.
    blocks: Vec<V::Element>,
}

impl<V: View> Markdown<V> {
    /// Render the given markdown source into a new widget.
    ///
    /// Uses [`default_options`] (CommonMark + GFM tables, strikethrough, task
    /// lists). For custom options, use [`Markdown::new_with`].
    pub fn new(src: &str) -> Self {
        Self::new_with(src, default_options())
    }

    /// Render the given markdown source with the given parser options.
    pub fn new_with(src: &str, opts: pulldown_cmark::Options) -> Self {
        let root = V::Element::new("div");
        root.set_property("class", "mogwai-md");
        let blocks = builder::children::<V>(src, opts);
        for block in &blocks {
            root.append_child(block);
        }
        Self { root, blocks }
    }

    /// Replace the rendered content with a new markdown source.
    ///
    /// In v0.1 this performs a wholesale rebuild: it removes all existing child
    /// nodes from the root `<div>` and appends a freshly parsed tree. No
    /// diffing is performed.
    pub fn set_source(&mut self, src: &str) {
        self.set_source_with(src, default_options());
    }

    /// Replace the rendered content with a new source and custom options.
    pub fn set_source_with(&mut self, src: &str, opts: pulldown_cmark::Options) {
        for block in &self.blocks {
            self.root.remove_child(block);
        }
        self.blocks = builder::children::<V>(src, opts);
        for block in &self.blocks {
            self.root.append_child(block);
        }
    }
}

impl<V: View> Default for Markdown<V> {
    fn default() -> Self {
        Self::new("")
    }
}
