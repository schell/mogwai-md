//! Event-stream to view-tree builder.
//!
//! Walks a `pulldown_cmark::Parser` event stream and produces a `Vec` of
//! top-level block elements. The walker maintains a stack of open containers
//! (one per `Event::Start` whose tag is a block element) so that inline
//! content is appended to the innermost open element. Inline content
//! (emphasis, strong, code, links, images) is handled by [`inline`].

use mogwai::prelude::*;
use pulldown_cmark::{Alignment, Event, Tag, TagEnd};

use crate::inline;

/// The parser options used by [`crate::Markdown::new`]: CommonMark + GFM
/// tables, strikethrough, and task lists.
pub fn default_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options as O;
    O::ENABLE_TABLES | O::ENABLE_STRIKETHROUGH | O::ENABLE_TASKLISTS
}

/// Parse `src` with `opts` and append the resulting view tree to `parent`.
///
/// Convenience wrapper around [`children`] that appends each top-level block
/// to `parent`. Use [`children`] directly when you need to track the appended
/// nodes (e.g. for removal later).
pub fn append_children<V: View>(parent: &V::Element, src: &str, opts: pulldown_cmark::Options) {
    for block in children::<V>(src, opts) {
        parent.append_child(&block);
    }
}

/// Parse `src` with `opts` and return the top-level block elements.
///
/// Each returned element is a block-level node (`<p>`, `<h1>`-`<h6>`,
/// `<ul>`/`<ol>`, `<blockquote>`, `<pre>`, `<table>`, `<hr>`, etc). Inline
/// content is nested inside the block that contains it.
pub fn children<V: View>(src: &str, opts: pulldown_cmark::Options) -> Vec<V::Element> {
    let parser = pulldown_cmark::Parser::new_ext(src, opts);
    let mut walker = Walker::<V>::new();
    for event in parser {
        walker.handle(event);
    }
    walker.top_level
}

struct Walker<V: View> {
    /// Completed top-level block elements, in document order.
    top_level: Vec<V::Element>,
    /// Stack of open block/inline elements. The top is the element that the
    /// next text/inline event should append to.
    stack: Vec<V::Element>,
    /// Per-open-image alt-text accumulator.
    image_alt: Option<String>,
    /// Per-open-heading text accumulator (for slugified `id`).
    heading_text: Option<String>,
    /// Column alignments for the current table, in column order. Reset on
    /// `TagEnd::Table`.
    table_alignments: Vec<Alignment>,
    /// Current column index within the current table row.
    table_column: usize,
}

impl<V: View> Walker<V> {
    fn new() -> Self {
        Self {
            top_level: Vec::new(),
            stack: Vec::new(),
            image_alt: None,
            heading_text: None,
            table_alignments: Vec::new(),
            table_column: 0,
        }
    }

    /// The element that inline content should be appended to. Falls back to
    /// the top-level context (returning `None` means the event is dropped).
    fn current(&self) -> Option<&V::Element> {
        self.stack.last()
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(end) => self.end_tag(end),
            Event::Text(s) => self.text(s.into_string()),
            Event::Code(s) => self.inline_code(s.into_string()),
            Event::SoftBreak => self.text("\n".to_string()),
            Event::HardBreak => self.br(),
            Event::TaskListMarker(checked) => self.tasklist_marker(checked),
            Event::FootnoteReference(s) => self.text(s.into_string()),
            Event::Html(s) | Event::InlineHtml(s) => {
                // mogwai-md never emits raw HTML. We deliberately drop it:
                // because we build a view tree from typed elements, there is
                // no XSS surface, and a sanitization policy is unnecessary.
                let _ = s;
            }
            Event::Rule => self.horizontal_rule(),
            Event::InlineMath(s) | Event::DisplayMath(s) => {
                // Math is out of scope for v0.1. Render the source as plain
                // text so the content is at least visible.
                self.text(s.into_string());
            }
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        let el = match tag {
            Tag::Paragraph => V::Element::new("p"),
            Tag::Heading { level, .. } => {
                let el = V::Element::new(heading_tag_name(level));
                self.heading_text = Some(String::new());
                el
            }
            Tag::BlockQuote(_) => V::Element::new("blockquote"),
            Tag::CodeBlock(kind) => {
                let pre = V::Element::new("pre");
                let code = V::Element::new("code");
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    let lang = lang.into_string();
                    if !lang.is_empty() {
                        code.set_property("class", format!("language-{lang}"));
                    }
                }
                pre.append_child(&code);
                // Push `pre` as the block (so it lands in top_level/parent),
                // then push `code` as the append target for text events. We
                // mark this by pushing both and popping two on End(CodeBlock).
                self.push_block(pre);
                self.stack.push(code);
                return;
            }
            Tag::HtmlBlock => {
                // We drop the contents (raw HTML is ignored). Push a sentinel
                // so the matching End(HtmlBlock) pops cleanly.
                let el = V::Element::new("template");
                self.push_block(el);
                return;
            }
            Tag::List(start) => {
                if start.is_some() {
                    V::Element::new("ol")
                } else {
                    V::Element::new("ul")
                }
            }
            Tag::Item => V::Element::new("li"),
            Tag::Emphasis => V::Element::new("em"),
            Tag::Strong => V::Element::new("strong"),
            Tag::Strikethrough => V::Element::new("del"),
            Tag::Superscript => V::Element::new("sup"),
            Tag::Subscript => V::Element::new("sub"),
            Tag::Link {
                dest_url, title, ..
            } => {
                let el = V::Element::new("a");
                el.set_property("href", dest_url.as_ref());
                let title = title.as_ref().to_string();
                if !title.is_empty() {
                    el.set_property("title", title);
                }
                if is_external_url(dest_url.as_ref()) {
                    el.set_property("rel", "nofollow noopener");
                    el.set_property("target", "_blank");
                }
                el
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                let el = V::Element::new("img");
                el.set_property("src", dest_url.as_ref());
                let title = title.as_ref().to_string();
                if !title.is_empty() {
                    el.set_property("title", title);
                }
                self.image_alt = Some(String::new());
                el
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.table_column = 0;
                V::Element::new("table")
            }
            Tag::TableHead => {
                self.table_column = 0;
                let thead = V::Element::new("thead");
                let tr = V::Element::new("tr");
                thead.append_child(&tr);
                // Push thead as block, tr as append target.
                self.push_block(thead);
                self.stack.push(tr);
                return;
            }
            Tag::TableRow => {
                self.table_column = 0;
                V::Element::new("tr")
            }
            Tag::TableCell => {
                let el = V::Element::new("td");
                if let Some(align) = self.table_alignments.get(self.table_column) {
                    if let Some(css) = alignment_css(align) {
                        el.set_style("text-align", css);
                    }
                }
                self.table_column += 1;
                el
            }
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {
                // pulldown-cmark emits these only with ENABLE_DEFINITION_LIST.
                // We don't enable that flag in default_options, but handle
                // gracefully in case a caller passes custom opts.
                V::Element::new("div")
            }
            Tag::FootnoteDefinition(_) => V::Element::new("div"),
            Tag::MetadataBlock(_) => {
                // YAML/TOML metadata block: drop the contents.
                let el = V::Element::new("template");
                self.push_block(el);
                return;
            }
        };

        self.push_block(el);
    }

    fn end_tag(&mut self, end: TagEnd) {
        match end {
            TagEnd::CodeBlock => {
                // Pop code, then pop pre.
                self.stack.pop();
                self.stack.pop();
            }
            TagEnd::TableHead => {
                // Pop tr, then pop thead.
                self.stack.pop();
                self.stack.pop();
            }
            TagEnd::Table => {
                self.stack.pop();
                self.table_alignments.clear();
            }
            TagEnd::Heading(_) => {
                if let Some(el) = self.stack.pop() {
                    if let Some(text) = self.heading_text.take() {
                        let slug = crate::slug::slugify(&text);
                        if !slug.is_empty() {
                            el.set_property("id", slug);
                        }
                    }
                }
            }
            TagEnd::Image => {
                if let Some(el) = self.stack.pop() {
                    if let Some(alt) = self.image_alt.take() {
                        if !alt.is_empty() {
                            el.set_property("alt", alt);
                        }
                    }
                }
            }
            _ => {
                self.stack.pop();
            }
        }
    }

    fn text(&mut self, s: String) {
        if let Some(heading) = self.heading_text.as_mut() {
            heading.push_str(&s);
        }
        if let Some(alt) = self.image_alt.as_mut() {
            alt.push_str(&s);
        }
        if let Some(el) = self.current() {
            let text = V::Text::new(s);
            el.append_child(&text);
        }
    }

    fn inline_code(&mut self, s: String) {
        if let Some(el) = self.current() {
            inline::code::<V>(el, s);
        }
    }

    fn br(&mut self) {
        if let Some(el) = self.current() {
            let br = V::Element::new("br");
            el.append_child(&br);
        }
    }

    fn horizontal_rule(&mut self) {
        let hr = V::Element::new("hr");
        if let Some(parent) = self.current() {
            parent.append_child(&hr);
        } else {
            // `<hr>` is a void element — it must never be pushed onto the
            // stack (there is no matching `TagEnd::Rule`, so it would never
            // be popped and every subsequent block would nest inside it,
            // inheriting the hr's UA styles). Promote it straight to
            // top-level instead.
            self.top_level.push(hr);
        }
    }

    fn tasklist_marker(&mut self, checked: bool) {
        if let Some(el) = self.current() {
            let input = V::Element::new("input");
            input.set_property("type", "checkbox");
            input.set_property("disabled", "disabled");
            if checked {
                input.set_property("checked", "checked");
            }
            el.append_child(&input);
        }
    }

    /// Push `el` as a block: append it to the current parent (or promote it to
    /// top-level if the stack is empty), then push it onto the stack.
    fn push_block(&mut self, el: V::Element) {
        if let Some(parent) = self.stack.last() {
            parent.append_child(&el);
        } else {
            self.top_level.push(el.clone());
        }
        self.stack.push(el);
    }
}

fn heading_tag_name(level: pulldown_cmark::HeadingLevel) -> &'static str {
    use pulldown_cmark::HeadingLevel as L;
    match level {
        L::H1 => "h1",
        L::H2 => "h2",
        L::H3 => "h3",
        L::H4 => "h4",
        L::H5 => "h5",
        L::H6 => "h6",
    }
}

fn alignment_css(align: &Alignment) -> Option<&'static str> {
    match align {
        Alignment::None => None,
        Alignment::Left => Some("left"),
        Alignment::Center => Some("center"),
        Alignment::Right => Some("right"),
    }
}

/// True if `url` is an absolute URL (has a scheme), indicating it points
/// outside the current document.
fn is_external_url(url: &str) -> bool {
    url.contains("://") || url.starts_with("//")
}

#[cfg(test)]
mod test {
    use super::*;
    use mogwai::ssr::{Ssr, SsrElement};

    fn render(src: &str) -> String {
        let blocks = children::<Ssr>(src, default_options());
        let root = SsrElement::new("div");
        for block in &blocks {
            root.append_child(block);
        }
        root.html_string()
    }

    #[test]
    fn paragraph() {
        let html = render("Hello, world.");
        assert!(html.contains("<p>Hello, world.</p>"));
    }

    #[test]
    fn heading_gets_slug_id() {
        let html = render("# Hello World");
        assert!(
            html.contains(r#"id="hello-world"#),
            "expected slugified id, got: {html}"
        );
    }

    #[test]
    fn emphasis_and_strong() {
        let html = render("*em* **strong**");
        assert!(html.contains("<em>em</em>"));
        assert!(html.contains("<strong>strong</strong>"));
    }

    #[test]
    fn strikethrough() {
        let html = render("~~deleted~~");
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn inline_code() {
        let html = render("`code`");
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn fenced_code_block_gets_language_class() {
        let src = "```rust\nfn main() {}\n```\n";
        let html = render(src);
        assert!(
            html.contains(r#"class="language-rust""#),
            "expected language-rust class, got: {html}"
        );
        assert!(html.contains("<pre>"));
    }

    #[test]
    fn link_external_gets_rel_and_target() {
        let html = render("[example](https://example.com)");
        assert!(
            html.contains(r#"rel="nofollow noopener""#),
            "expected rel attribute, got: {html}"
        );
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(">example</a>"));
    }

    #[test]
    fn link_internal_no_rel() {
        let html = render("[about](/about)");
        assert!(!html.contains("rel="));
        assert!(html.contains(r#"href="/about""#));
    }

    #[test]
    fn link_fragment_is_in_page() {
        // In-app fragment links (`#beads/{id}` and friends) navigate the
        // current page — the host's hash routing handles them — so they
        // must never get the external-link treatment.
        let html = render("[bead](#beads/schell-l83)");
        assert!(html.contains(r##"href="#beads/schell-l83""##));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
    }

    #[test]
    fn unordered_list() {
        let html = render("- one\n- two\n");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>one</li>"));
        assert!(html.contains("<li>two</li>"));
    }

    #[test]
    fn ordered_list() {
        let html = render("1. first\n2. second\n");
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>first</li>"));
    }

    #[test]
    fn tasklist() {
        let html = render("- [x] done\n- [ ] todo\n");
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"checked="checked""#));
    }

    #[test]
    fn blockquote() {
        let html = render("> wisdom\n");
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("wisdom"));
    }

    #[test]
    fn horizontal_rule() {
        let html = render("---\n");
        assert!(html.contains("<hr"));
    }

    #[test]
    fn content_after_horizontal_rule_is_not_nested() {
        // Regression: `Event::Rule` must not leave the `<hr>` on the
        // walker's open-element stack. If it does, the paragraph below
        // becomes a child of the `<hr>`, inheriting its UA styles
        // (gray/half-transparent) in the browser, and SSR emits a
        // malformed `<hr><p>below</p></hr>`.
        let html = render("above\n\n---\n\nbelow\n");
        assert!(html.contains("<hr />"));
        assert!(html.contains("<p>above</p>"));
        assert!(html.contains("<p>below</p>"));
        assert!(
            !html.contains("<hr><p>below</p>"),
            "below must not nest inside <hr>, got: {html}"
        );
        assert!(
            !html.contains("<hr /><p>below</p></hr>"),
            "<hr> must be self-closing, got: {html}"
        );
    }

    #[test]
    fn table() {
        let src = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let html = render(src);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn raw_html_is_dropped() {
        let html = render("<script>alert(1)</script>");
        assert!(!html.contains("<script"));
    }

    #[test]
    fn image_alt_text() {
        let html = render("![alt text](/img.png \"title\")");
        assert!(
            html.contains(r#"alt="alt text""#),
            "expected alt attribute, got: {html}"
        );
        assert!(html.contains(r#"title="title""#));
        assert!(html.contains(r#"src="/img.png""#));
    }
}
