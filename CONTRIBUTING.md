# Contributing

Thanks for your interest in contributing to gpui_maplibre.

## Ways to help

- Report bugs and regressions
- Improve documentation and examples

## Development setup

This repository is a Rust crate with private WebView assets and optional
JavaScript bridge tests. Typical tasks:

- `cargo metadata --format-version 1`
- `cargo fmt --all -- --check`
- `cargo test --no-default-features`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `npm test --prefix js-tests` once JavaScript tests are present
- `cargo check --features gpui-webview` where platform dependencies permit

## Pull request checklist

- Keep changes focused and well-scoped
- Add or update tests when behavior changes
- Keep public APIs documented
- Avoid introducing new unsafe code
- Keep WebView, HTML, and JavaScript bridge details private to the crate
- Keep examples and core APIs generalized, not application-specific

## Code style

- Use idiomatic Rust
- Prefer small, composable helpers
- Favor clear, explicit APIs over cleverness
- Prefer typed Rust commands, events, handles, and errors at public boundaries

## License

By contributing, you agree that your contributions are released under the
project license (MIT OR Apache-2.0).
