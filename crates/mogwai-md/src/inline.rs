//! Inline content helpers.
//!
//! These are small helpers used by [`crate::builder`] for inline content that
//! is awkward to inline in the match arms (currently just inline code, which
//! is a `<code>` element wrapping a text node). Keeping them here lets the
//! builder module stay focused on the event-stream walker.

use mogwai::prelude::*;

/// Append an inline `<code>` element containing `s` to `parent`.
pub fn code<V: View>(parent: &V::Element, s: String) {
    let code = V::Element::new("code");
    code.append_child(V::Text::new(s));
    parent.append_child(&code);
}
