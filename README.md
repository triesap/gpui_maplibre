# gpui_maplibre

MapLibre GL JS wrapper for GPUI in Rust

## Goals

- Render and control a MapLibre map in GPUI apps.
- Provide an ergonomic Rust API for style, camera, interaction, and events.
- Keep core non-opinionated and leave control UI to application code.
- Use a private GPUI WebView bridge for rendering, command dispatch, and event delivery.

## Renderer source

Map rendering is provided by MapLibre GL JS in a private GPUI WebView.

## Contributing

See `CONTRIBUTING.md`.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
