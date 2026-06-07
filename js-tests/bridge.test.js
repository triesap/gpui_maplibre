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
