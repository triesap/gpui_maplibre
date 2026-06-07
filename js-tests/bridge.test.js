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
    return {
        messages,
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
