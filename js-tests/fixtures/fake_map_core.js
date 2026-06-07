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

export function add_geojson_source(handle, source_id, geojson, promote_id) {
    record("add_geojson_source", { handle, source_id, geojson, promote_id });
}

export function update_geojson_source(handle, source_id, geojson) {
    record("update_geojson_source", { handle, source_id, geojson });
}

export function remove_source(handle, source_id) {
    record("remove_source", { handle, source_id });
}

export function add_layer(handle, layer_id, layer_spec, before_id) {
    record("add_layer", { handle, layer_id, layer_spec, before_id });
}

export function remove_layer(handle, layer_id) {
    record("remove_layer", { handle, layer_id });
}

export function set_layout_property(handle, layer_id, property_name, value) {
    record("set_layout_property", { handle, layer_id, property_name, value });
}

export function set_paint_property(handle, layer_id, property_name, value) {
    record("set_paint_property", { handle, layer_id, property_name, value });
}

export function set_filter(handle, layer_id, filter) {
    record("set_filter", { handle, layer_id, filter });
}

export function set_layer_zoom_range(handle, layer_id, min_zoom, max_zoom) {
    record("set_layer_zoom_range", { handle, layer_id, min_zoom, max_zoom });
}

export function set_feature_state(handle, source_id, source_layer, feature_id, state) {
    record("set_feature_state", {
        handle,
        source_id,
        source_layer,
        feature_id,
        state,
    });
}

export function set_terrain(handle, terrain) {
    record("set_terrain", { handle, terrain });
}

export function set_fog(handle, fog) {
    record("set_fog", { handle, fog });
}

export function set_light(handle, light) {
    record("set_light", { handle, light });
}

export function add_native_control(handle, control_kind, anchor, options) {
    record("add_native_control", { handle, control_kind, anchor, options });
    return 7;
}

export function remove_native_control(control_handle) {
    record("remove_native_control", { control_handle });
}

export function create_marker(handle, lng, lat, draggable, anchor, offset_x, offset_y, rotation) {
    record("create_marker", {
        handle,
        lng,
        lat,
        draggable,
        anchor,
        offset_x,
        offset_y,
        rotation,
    });
    return 2;
}

export function update_marker(marker_handle, lng, lat, draggable, anchor, offset_x, offset_y, rotation) {
    record("update_marker", {
        marker_handle,
        lng,
        lat,
        draggable,
        anchor,
        offset_x,
        offset_y,
        rotation,
    });
}

export function remove_marker(marker_handle) {
    record("remove_marker", { marker_handle });
}

export function create_popup(
    handle,
    lng,
    lat,
    html,
    close_button,
    close_on_click,
    anchor,
    offset_x,
    offset_y,
    max_width,
) {
    record("create_popup", {
        handle,
        lng,
        lat,
        html,
        close_button,
        close_on_click,
        anchor,
        offset_x,
        offset_y,
        max_width,
    });
    return 3;
}

export function create_popup_text(
    handle,
    lng,
    lat,
    text,
    close_button,
    close_on_click,
    anchor,
    offset_x,
    offset_y,
    max_width,
) {
    record("create_popup_text", {
        handle,
        lng,
        lat,
        text,
        close_button,
        close_on_click,
        anchor,
        offset_x,
        offset_y,
        max_width,
    });
    return 4;
}

export function update_popup(popup_handle, lng, lat, html, offset_x, offset_y, max_width) {
    record("update_popup", {
        popup_handle,
        lng,
        lat,
        html,
        offset_x,
        offset_y,
        max_width,
    });
}

export function update_popup_text(popup_handle, lng, lat, text, offset_x, offset_y, max_width) {
    record("update_popup_text", {
        popup_handle,
        lng,
        lat,
        text,
        offset_x,
        offset_y,
        max_width,
    });
}

export function remove_popup(popup_handle) {
    record("remove_popup", { popup_handle });
}

export function register_on_map_events(handle, callback) {
    record("register_on_map_events", { handle, callback });
    return 4;
}

export function unregister_on_map_events(handle) {
    record("unregister_on_map_events", { handle });
}

export function register_on_layer_events(handle, layer_id, callback) {
    record("register_on_layer_events", { handle, layer_id, callback });
    return 5;
}

export function unregister_on_layer_events(handle, layer_id) {
    record("unregister_on_layer_events", { handle, layer_id });
}

export function register_on_marker_drag_events(marker_handle, callback) {
    record("register_on_marker_drag_events", { marker_handle, callback });
}

export function unregister_on_marker_drag_events(marker_handle) {
    record("unregister_on_marker_drag_events", { marker_handle });
}

export function register_on_popup_events(popup_handle, callback) {
    record("register_on_popup_events", { popup_handle, callback });
}

export function unregister_on_popup_events(popup_handle) {
    record("unregister_on_popup_events", { popup_handle });
}
