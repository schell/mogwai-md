# mogwai-md

A markdown rendering widget for **mogwai**, parsed with *pulldown-cmark*.

## Inline content

This paragraph has **strong**, *emphasis*, ~~strikethrough~~, and `inline code`.
Links work too: [example.com](https://example.com) (external) and
[about](/about) (internal).

## Lists

Unordered:

- one
- two
- three

Ordered:

1. first
2. second
3. third

Task list:

- [x] render markdown
- [x] support GFM
- [ ] syntax highlighting (out of scope for v0.1)

## Code block

```rust
fn main() {
    println!("Hello, mogwai-md!");
}
```

## Blockquote

> Reality is that which, when you stop believing in it, doesn't go away.
> -- Philip K. Dick

## Table

| feature     | status | notes                          |
|-------------|:------:|--------------------------------|
| paragraphs  | done   |                                |
| headings    | done   | slugified `id` for anchors     |
| tables      | done   | GFM, with alignment            |
| syntax-hl   | no     | planned for v0.2               |

## Horizontal rule

---

That's all for now.