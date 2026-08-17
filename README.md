# mogwai-md

A markdown rendering widget for [mogwai](https://github.com/schell/mogwai).

`mogwai-md` parses markdown using [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
and lowers the event stream directly into a mogwai view tree. Because it builds
real DOM elements (not an HTML string), the same widget works in WASM
(`mogwai::web::Web`) and server-side rendering (`mogwai::ssr::Ssr`).

## Status

v0.1 — work in progress.

Scope of v0.1:

- CommonMark + GFM (tables, task lists, strikethrough)
- Headings with slugified `id` attributes for anchor links
- Links, images, blockquotes, fenced code blocks with `language-` classes
- Wholesale rebuild on `set_source` (no diffing)

Out of scope for v0.1: syntax highlighting, CSS, HTML sanitization (we never
emit raw HTML), diff-based updates, math, custom render hooks.

## Quickstart

### Trunk demo

```sh
trunk serve
```

Open <http://127.0.0.1:8080>. Trunk rebuilds on save.

### Use in a mogwai app

```rust
use mogwai::prelude::*;
use mogwai::web::Web;
use mogwai_md::Markdown;

let md = Markdown::<Web>::new("# Hello, world\n\nThis is **mogwai-md**.");
mogwai::web::body().append_child(&md);
```

## License

MIT OR Apache-2.0.