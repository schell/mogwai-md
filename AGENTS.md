# AGENTS.md — mogwai-md

A markdown rendering widget for mogwai. Uses pulldown-cmark's event stream to
build a mogwai view tree (works for both `mogwai::web::Web` and
`mogwai::ssr::Ssr`).

## Layout

```
crates/mogwai-md/   # the library (rlib)
examples/demo/      # trunk-servable WASM demo (cdylib)
```

## Build / test / lint

```sh
cargo check                         # typecheck everything
cargo test -p mogwai-md             # host unit tests (uses Ssr view)
cargo clippy --all-targets -- -D warnings
cargo +nightly fmt --check          # rustfmt.toml uses unstable features
trunk serve                         # demo at http://127.0.0.1:8080
```

WASM smoke test (optional, requires a browser):

```sh
wasm-pack test --headless --chrome crates/mogwai-md --features wasm-test
```

## Conventions

- `#![forbid(unsafe_code)]` in the library crate.
- Errors: `snafu` with span info (not `anyhow`/`thiserror`) — when needed.
  v0.1's public API is infallible (the parser is infallible and view
  construction does not return `Result`).
- Format with `cargo +nightly fmt` — `rustfmt.toml` uses unstable features
  (`imports_granularity`, `format_strings`, `wrap_comments`); stable `cargo
  fmt` silently skips them.
- Max width: 100 chars.
- Tests: inline `#[cfg(test)] mod test { ... }` within modules.

## Beads

This repo has **no per-repo `.beads`**. All issues live in the personal DB at
`~/.beads` (prefix `schell-`), inherited via ancestor walk from `~/code`. Never
run `bd init` here. From inside this git repo, `bd where` fails (bd's worktree
logic short-circuits the ancestor walk) — prefix bd commands with
`BEADS_DIR=/Users/schell/.beads`, or run `bd` from `~/code` (non-git dir).

Epic for v0.1: `schell-4xh` with children `schell-4xh.1` through `schell-4xh.6`.

## AI Disclosure

Commits authored with AI must use the format:

```
{human-author} with {llm-name} {llm-version} <{human-email}>
```

Set via: `git commit --amend --author "{human-author} with {llm-name} {llm-version} <{human-email}>"`

See [NLnet's AI Disclosure Policy](https://nlnet.nl/foundation/policies/generativeAI/).

## Non-interactive shell flags

Use `cp -f`, `mv -f`, `rm -f`, `rm -rf` — `cp`/`mv`/`rm` may be aliased to
interactive mode on this machine.