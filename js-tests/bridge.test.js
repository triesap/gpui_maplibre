import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import * as fakeMapCore from "./fixtures/fake_map_core.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const bridgeSource = await readFile(
    resolve(__dirname, "../crates/gpui_maplibre/assets/bridge.js"),
    "utf8",
);
const fakeMapCoreUrl = pathToFileURL(
    resolve(__dirname, "fixtures/fake_map_core.js"),
).href;
const bridgeTestSource = bridgeSource.replace(
    'import * as mapCore from "./map_core.js";',
    `import * as mapCore from ${JSON.stringify(fakeMapCoreUrl)};`,
);
const bridgeModuleUrl = `data:text/javascript;base64,${Buffer.from(bridgeTestSource).toString(
    "base64",
)}`;
const { dispatch, install_bridge, post, post_dom_ready } = await import(bridgeModuleUrl);

function test_target() {
    const messages = [];
    const mapElement = { id: "map" };
    return {
        messages,
        mapElement,
        document: {
            getElementById(id) {
                return id === "map" ? mapElement : null;
            },
        },
        ipc: {
            postMessage(message) {
                messages.push(JSON.parse(message));
            },
        },
    };
}

test("post serializes payloads through ipc.postMessage", () => {
    const target = test_target();

    const message = post({ type: "dom_ready" }, target);

    assert.equal(message, "{\"type\":\"dom_ready\"}");
    assert.deepEqual(target.messages, [{ type: "dom_ready" }]);
});

test("dispatch init calls map core and posts initialized handle", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    const result = dispatch(
        {
            type: "init",
            options: {
                style_url: "maplibre://styles/basic",
                center_lng: -123.1,
                center_lat: 49.2,
                zoom: 11,
            },
        },
        target,
    );

    assert.deepEqual(result, { ok: true });
    assert.deepEqual(target.messages, [{ type: "initialized", handle: 1 }]);
    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "init_map",
            payload: {
                container: target.mapElement,
                options: {
                    style_url: "maplibre://styles/basic",
                    center_lng: -123.1,
                    center_lat: 49.2,
                    zoom: 11,
                },
            },
        },
    ]);
});

test("dispatch lifecycle and style commands call map core", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    assert.deepEqual(dispatch({ type: "resize", handle: 1 }, target), { ok: true });
    assert.deepEqual(
        dispatch({ type: "set_style", handle: 1, style_url: "maplibre://styles/dark" }, target),
        { ok: true },
    );
    assert.deepEqual(dispatch({ type: "destroy", handle: 1 }, target), { ok: true });

    assert.deepEqual(fakeMapCore.recorded_calls(), [
        { name: "resize_map", payload: { handle: 1 } },
        {
            name: "set_style",
            payload: {
                handle: 1,
                style_url: "maplibre://styles/dark",
            },
        },
        { name: "destroy_map", payload: { handle: 1 } },
    ]);
});

test("dispatch camera commands call map core with expected arguments", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    for (const command of [
        {
            type: "fly_to",
            handle: 1,
            lng: -123.1,
            lat: 49.2,
            zoom: 11,
            duration_ms: 750,
        },
        {
            type: "jump_to",
            handle: 1,
            lng: -123.2,
            lat: 49.3,
            zoom: 12,
            bearing: 15,
            pitch: 30,
        },
        {
            type: "ease_to",
            handle: 1,
            lng: -123.3,
            lat: 49.4,
            zoom: 13,
            bearing: null,
            pitch: 25,
            duration_ms: 500,
        },
        {
            type: "fit_bounds",
            handle: 1,
            west: -124,
            south: 48,
            east: -122,
            north: 50,
            padding: 24,
            duration_ms: null,
            max_zoom: 14,
        },
    ]) {
        assert.deepEqual(dispatch(command, target), { ok: true });
    }

    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "fly_to",
            payload: {
                handle: 1,
                lng: -123.1,
                lat: 49.2,
                zoom: 11,
                duration_ms: 750,
            },
        },
        {
            name: "jump_to",
            payload: {
                handle: 1,
                lng: -123.2,
                lat: 49.3,
                zoom: 12,
                bearing: 15,
                pitch: 30,
            },
        },
        {
            name: "ease_to",
            payload: {
                handle: 1,
                lng: -123.3,
                lat: 49.4,
                zoom: 13,
                bearing: null,
                pitch: 25,
                duration_ms: 500,
            },
        },
        {
            name: "fit_bounds",
            payload: {
                handle: 1,
                west: -124,
                south: 48,
                east: -122,
                north: 50,
                padding: 24,
                duration_ms: null,
                max_zoom: 14,
            },
        },
    ]);
});

test("dispatch source commands pass JSON payloads through", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    const sourceSpec = { type: "vector", url: "mapbox://tiles" };
    const emptyCollection = { type: "FeatureCollection", features: [] };
    const updatedCollection = {
        type: "FeatureCollection",
        features: [{ type: "Feature", id: "se-1" }],
    };

    for (const command of [
        {
            type: "add_source",
            handle: 1,
            source_id: "tiles",
            source_spec: sourceSpec,
        },
        {
            type: "add_geojson_source",
            handle: 1,
            source_id: "places",
            geojson: emptyCollection,
            promote_id: "id",
        },
        {
            type: "update_geojson_source",
            handle: 1,
            source_id: "places",
            geojson: updatedCollection,
        },
        {
            type: "remove_source",
            handle: 1,
            source_id: "tiles",
        },
    ]) {
        assert.deepEqual(dispatch(command, target), { ok: true });
    }

    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "add_source",
            payload: {
                handle: 1,
                source_id: "tiles",
                source_spec: sourceSpec,
            },
        },
        {
            name: "add_geojson_source",
            payload: {
                handle: 1,
                source_id: "places",
                geojson: emptyCollection,
                promote_id: "id",
            },
        },
        {
            name: "update_geojson_source",
            payload: {
                handle: 1,
                source_id: "places",
                geojson: updatedCollection,
            },
        },
        {
            name: "remove_source",
            payload: {
                handle: 1,
                source_id: "tiles",
            },
        },
    ]);
});

test("dispatch layer and property commands pass style-spec values through", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    const layerSpec = { type: "circle", source: "places" };
    const filter = ["==", ["get", "kind"], "harbor"];

    for (const command of [
        {
            type: "add_layer",
            handle: 1,
            layer_id: "places-circle",
            layer_spec: layerSpec,
            before_id: "labels",
        },
        {
            type: "set_layout_property",
            handle: 1,
            layer_id: "places-circle",
            property_name: "visibility",
            value: "none",
        },
        {
            type: "set_paint_property",
            handle: 1,
            layer_id: "places-circle",
            property_name: "circle-color",
            value: "#2b6cb0",
        },
        {
            type: "set_filter",
            handle: 1,
            layer_id: "places-circle",
            filter,
        },
        {
            type: "set_layer_zoom_range",
            handle: 1,
            layer_id: "places-circle",
            min_zoom: 4,
            max_zoom: 12,
        },
        {
            type: "remove_layer",
            handle: 1,
            layer_id: "places-circle",
        },
    ]) {
        assert.deepEqual(dispatch(command, target), { ok: true });
    }

    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "add_layer",
            payload: {
                handle: 1,
                layer_id: "places-circle",
                layer_spec: layerSpec,
                before_id: "labels",
            },
        },
        {
            name: "set_layout_property",
            payload: {
                handle: 1,
                layer_id: "places-circle",
                property_name: "visibility",
                value: "none",
            },
        },
        {
            name: "set_paint_property",
            payload: {
                handle: 1,
                layer_id: "places-circle",
                property_name: "circle-color",
                value: "#2b6cb0",
            },
        },
        {
            name: "set_filter",
            payload: {
                handle: 1,
                layer_id: "places-circle",
                filter,
            },
        },
        {
            name: "set_layer_zoom_range",
            payload: {
                handle: 1,
                layer_id: "places-circle",
                min_zoom: 4,
                max_zoom: 12,
            },
        },
        {
            name: "remove_layer",
            payload: {
                handle: 1,
                layer_id: "places-circle",
            },
        },
    ]);
});

test("dispatch feature-state and scene commands pass values through", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    for (const command of [
        {
            type: "set_feature_state",
            handle: 1,
            source_id: "places",
            source_layer: "settlements",
            feature_id: "se-1",
            state: { selected: true },
        },
        {
            type: "set_terrain",
            handle: 1,
            terrain: { source: "terrain", exaggeration: 1.2 },
        },
        {
            type: "set_fog",
            handle: 1,
            fog: null,
        },
        {
            type: "set_light",
            handle: 1,
            light: { anchor: "viewport", intensity: 0.4 },
        },
    ]) {
        assert.deepEqual(dispatch(command, target), { ok: true });
    }

    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "set_feature_state",
            payload: {
                handle: 1,
                source_id: "places",
                source_layer: "settlements",
                feature_id: "se-1",
                state: { selected: true },
            },
        },
        {
            name: "set_terrain",
            payload: {
                handle: 1,
                terrain: { source: "terrain", exaggeration: 1.2 },
            },
        },
        {
            name: "set_fog",
            payload: {
                handle: 1,
                fog: null,
            },
        },
        {
            name: "set_light",
            payload: {
                handle: 1,
                light: { anchor: "viewport", intensity: 0.4 },
            },
        },
    ]);
});

test("dispatch controls and marker commands post acknowledgements", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    assert.deepEqual(
        dispatch(
            {
                type: "add_native_control",
                request_id: 10,
                handle: 1,
                kind: "navigation",
                anchor: "top_right",
                options: { showCompass: true },
            },
            target,
        ),
        { ok: true },
    );
    assert.deepEqual(
        dispatch({ type: "remove_native_control", control_handle: 7 }, target),
        { ok: true },
    );
    assert.deepEqual(
        dispatch(
            {
                type: "create_marker",
                request_id: 11,
                handle: 1,
                options: {
                    lng: -123.1,
                    lat: 49.2,
                    draggable: true,
                    anchor: "bottom",
                    offset_x: 4,
                    offset_y: 8,
                    rotation: 45,
                },
            },
            target,
        ),
        { ok: true },
    );
    assert.deepEqual(
        dispatch(
            {
                type: "update_marker",
                marker_handle: 2,
                options: {
                    lng: -123.2,
                    lat: 49.3,
                    draggable: false,
                    anchor: null,
                    offset_x: null,
                    offset_y: null,
                    rotation: null,
                },
            },
            target,
        ),
        { ok: true },
    );
    assert.deepEqual(dispatch({ type: "remove_marker", marker_handle: 2 }, target), {
        ok: true,
    });

    assert.deepEqual(target.messages, [
        {
            type: "native_control_created",
            request_id: 10,
            control_handle: 7,
        },
        {
            type: "marker_created",
            request_id: 11,
            marker_handle: 2,
        },
    ]);
    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "add_native_control",
            payload: {
                handle: 1,
                control_kind: "navigation",
                anchor: "top_right",
                options: { showCompass: true },
            },
        },
        {
            name: "remove_native_control",
            payload: {
                control_handle: 7,
            },
        },
        {
            name: "create_marker",
            payload: {
                handle: 1,
                lng: -123.1,
                lat: 49.2,
                draggable: true,
                anchor: "bottom",
                offset_x: 4,
                offset_y: 8,
                rotation: 45,
            },
        },
        {
            name: "update_marker",
            payload: {
                marker_handle: 2,
                lng: -123.2,
                lat: 49.3,
                draggable: false,
                anchor: null,
                offset_x: null,
                offset_y: null,
                rotation: null,
            },
        },
        {
            name: "remove_marker",
            payload: {
                marker_handle: 2,
            },
        },
    ]);
});

test("dispatch popup commands choose safe text or trusted html paths", () => {
    const target = test_target();
    fakeMapCore.reset_calls();

    assert.deepEqual(
        dispatch(
            {
                type: "create_popup",
                request_id: 12,
                handle: 1,
                options: {
                    lng: -123.1,
                    lat: 49.2,
                    content: { kind: "text", value: "<b>plain text</b>" },
                    close_button: true,
                    close_on_click: false,
                    anchor: "top",
                    offset_x: 1,
                    offset_y: 2,
                    max_width: 320,
                },
            },
            target,
        ),
        { ok: true },
    );
    assert.deepEqual(
        dispatch(
            {
                type: "update_popup",
                popup_handle: 4,
                options: {
                    lng: -123.2,
                    lat: 49.3,
                    content: { kind: "trusted_html", value: "<strong>trusted</strong>" },
                    offset_x: null,
                    offset_y: null,
                    max_width: 400,
                },
            },
            target,
        ),
        { ok: true },
    );
    assert.deepEqual(dispatch({ type: "remove_popup", popup_handle: 4 }, target), {
        ok: true,
    });

    assert.deepEqual(target.messages, [
        {
            type: "popup_created",
            request_id: 12,
            popup_handle: 4,
        },
    ]);
    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "create_popup_text",
            payload: {
                handle: 1,
                lng: -123.1,
                lat: 49.2,
                text: "<b>plain text</b>",
                close_button: true,
                close_on_click: false,
                anchor: "top",
                offset_x: 1,
                offset_y: 2,
                max_width: 320,
            },
        },
        {
            name: "update_popup",
            payload: {
                popup_handle: 4,
                lng: -123.2,
                lat: 49.3,
                html: "<strong>trusted</strong>",
                offset_x: null,
                offset_y: null,
                max_width: 400,
            },
        },
        {
            name: "remove_popup",
            payload: {
                popup_handle: 4,
            },
        },
    ]);
});

test("post_dom_ready emits the dom_ready IPC event", () => {
    const target = test_target();

    post_dom_ready(target);

    assert.deepEqual(target.messages, [{ type: "dom_ready" }]);
});

test("dispatch shell handles dom_ready commands", () => {
    const target = test_target();

    const result = dispatch({ type: "dom_ready" }, target);

    assert.deepEqual(result, { ok: true });
    assert.deepEqual(target.messages, [{ type: "dom_ready" }]);
});

test("dispatch shell reports unknown commands as structured errors", () => {
    const target = test_target();

    const result = dispatch({ type: "not_real" }, target);

    assert.deepEqual(result, { ok: false });
    assert.equal(target.messages.length, 1);
    assert.equal(target.messages[0].type, "error");
    assert.equal(target.messages[0].context, "unknown_command");
    assert.match(target.messages[0].message, /not_real/);
});

test("install_bridge exposes dispatch and can host a fake map core", () => {
    const target = test_target();
    const bridge = install_bridge(target);
    bridge.map_core = fakeMapCore;
    fakeMapCore.reset_calls();

    const handle = target.__gpui_maplibre.map_core.init_map(
        { id: "map" },
        { style_url: "maplibre://styles/basic" },
    );

    assert.equal(handle, 1);
    assert.equal(typeof target.__gpui_maplibre.dispatch, "function");
    assert.deepEqual(fakeMapCore.recorded_calls(), [
        {
            name: "init_map",
            payload: {
                container: { id: "map" },
                options: { style_url: "maplibre://styles/basic" },
            },
        },
    ]);
});
