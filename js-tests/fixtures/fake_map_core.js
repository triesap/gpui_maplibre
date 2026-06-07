const calls = [];

export function reset_calls() {
    calls.length = 0;
}

export function recorded_calls() {
    return [...calls];
}

function record(name, payload = {}) {
    calls.push({ name, payload });
}

export function init_map(container, options) {
    record("init_map", { container, options });
    return 1;
}

export function destroy_map(handle) {
    record("destroy_map", { handle });
}

export function resize_map(handle) {
    record("resize_map", { handle });
}

export function set_style(handle, style_url) {
    record("set_style", { handle, style_url });
}

export function fly_to(handle, lng, lat, zoom, duration_ms) {
    record("fly_to", { handle, lng, lat, zoom, duration_ms });
}

export function jump_to(handle, lng, lat, zoom, bearing, pitch) {
    record("jump_to", { handle, lng, lat, zoom, bearing, pitch });
}

export function ease_to(handle, lng, lat, zoom, bearing, pitch, duration_ms) {
    record("ease_to", { handle, lng, lat, zoom, bearing, pitch, duration_ms });
}

export function fit_bounds(handle, west, south, east, north, padding, duration_ms, max_zoom) {
    record("fit_bounds", {
        handle,
        west,
        south,
        east,
        north,
        padding,
        duration_ms,
        max_zoom,
    });
}

export function add_source(handle, source_id, source_spec) {
    record("add_source", { handle, source_id, source_spec });
}

export function add_layer(handle, layer_id, layer_spec, before_id) {
    record("add_layer", { handle, layer_id, layer_spec, before_id });
}

export function create_marker(handle, options) {
    record("create_marker", { handle, options });
    return 2;
}

export function create_popup(handle, options) {
    record("create_popup", { handle, options });
    return 3;
}

export function register_on_map_events(handle, callback) {
    record("register_on_map_events", { handle, callback });
    return 4;
}

export function register_on_layer_events(handle, layer_id, callback) {
    record("register_on_layer_events", { handle, layer_id, callback });
    return 5;
}
