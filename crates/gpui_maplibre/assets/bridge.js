import * as mapCore from "./map_core.js";

const BRIDGE_KEY = "__gpui_maplibre";

function default_target() {
    return typeof window !== "undefined" ? window : globalThis;
}

function describe_error(error) {
    if (error instanceof Error) {
        return error.message;
    }
    if (typeof error === "string") {
        return error;
    }
    return "unknown bridge error";
}

function target_document(target) {
    if (target?.document !== undefined) {
        return target.document;
    }
    return typeof document !== "undefined" ? document : undefined;
}

function map_container(target) {
    return target_document(target)?.getElementById?.("map") ?? null;
}

function require_map_container(target) {
    const container = map_container(target);
    if (container === null) {
        throw new Error("missing #map container");
    }
    return container;
}

export function post(payload, target = default_target()) {
    const message = JSON.stringify(payload);
    if (typeof window !== "undefined" && target === window && window.ipc?.postMessage) {
        window.ipc.postMessage(message);
        return message;
    }
    if (target?.ipc?.postMessage) {
        target.ipc.postMessage(message);
        return message;
    }
    console.info("gpui_maplibre ipc", message);
    return message;
}

export function post_error(context, error, target = default_target()) {
    return post(
        {
            type: "error",
            context,
            message: describe_error(error),
        },
        target,
    );
}

export function post_dom_ready(target = default_target()) {
    return post({ type: "dom_ready" }, target);
}

export function dispatch(command, target = default_target()) {
    try {
        if (command === undefined || command === null || typeof command.type !== "string") {
            throw new Error("command.type is required");
        }

        if (command.type === "dom_ready") {
            post_dom_ready(target);
            return { ok: true };
        }

        if (command.type === "init") {
            const handle = mapCore.init_map(require_map_container(target), command.options ?? {});
            post({ type: "initialized", handle }, target);
            return { ok: true };
        }

        if (command.type === "destroy") {
            mapCore.destroy_map(command.handle);
            return { ok: true };
        }

        if (command.type === "resize") {
            mapCore.resize_map(command.handle);
            return { ok: true };
        }

        if (command.type === "set_style") {
            mapCore.set_style(command.handle, command.style_url);
            return { ok: true };
        }

        if (command.type === "fly_to") {
            mapCore.fly_to(
                command.handle,
                command.lng,
                command.lat,
                command.zoom,
                command.duration_ms,
            );
            return { ok: true };
        }

        if (command.type === "jump_to") {
            mapCore.jump_to(
                command.handle,
                command.lng,
                command.lat,
                command.zoom,
                command.bearing,
                command.pitch,
            );
            return { ok: true };
        }

        if (command.type === "ease_to") {
            mapCore.ease_to(
                command.handle,
                command.lng,
                command.lat,
                command.zoom,
                command.bearing,
                command.pitch,
                command.duration_ms,
            );
            return { ok: true };
        }

        if (command.type === "fit_bounds") {
            mapCore.fit_bounds(
                command.handle,
                command.west,
                command.south,
                command.east,
                command.north,
                command.padding,
                command.duration_ms,
                command.max_zoom,
            );
            return { ok: true };
        }

        if (command.type === "add_source") {
            mapCore.add_source(command.handle, command.source_id, command.source_spec);
            return { ok: true };
        }

        if (command.type === "add_geojson_source") {
            mapCore.add_geojson_source(
                command.handle,
                command.source_id,
                command.geojson,
                command.promote_id,
            );
            return { ok: true };
        }

        if (command.type === "update_geojson_source") {
            mapCore.update_geojson_source(command.handle, command.source_id, command.geojson);
            return { ok: true };
        }

        if (command.type === "remove_source") {
            mapCore.remove_source(command.handle, command.source_id);
            return { ok: true };
        }

        if (command.type === "add_layer") {
            mapCore.add_layer(
                command.handle,
                command.layer_id,
                command.layer_spec,
                command.before_id,
            );
            return { ok: true };
        }

        if (command.type === "remove_layer") {
            mapCore.remove_layer(command.handle, command.layer_id);
            return { ok: true };
        }

        if (command.type === "set_layout_property") {
            mapCore.set_layout_property(
                command.handle,
                command.layer_id,
                command.property_name,
                command.value,
            );
            return { ok: true };
        }

        if (command.type === "set_paint_property") {
            mapCore.set_paint_property(
                command.handle,
                command.layer_id,
                command.property_name,
                command.value,
            );
            return { ok: true };
        }

        if (command.type === "set_filter") {
            mapCore.set_filter(command.handle, command.layer_id, command.filter);
            return { ok: true };
        }

        if (command.type === "set_layer_zoom_range") {
            mapCore.set_layer_zoom_range(
                command.handle,
                command.layer_id,
                command.min_zoom,
                command.max_zoom,
            );
            return { ok: true };
        }

        if (command.type === "set_feature_state") {
            mapCore.set_feature_state(
                command.handle,
                command.source_id,
                command.source_layer,
                command.feature_id,
                command.state,
            );
            return { ok: true };
        }

        if (command.type === "set_terrain") {
            mapCore.set_terrain(command.handle, command.terrain);
            return { ok: true };
        }

        if (command.type === "set_fog") {
            mapCore.set_fog(command.handle, command.fog);
            return { ok: true };
        }

        if (command.type === "set_light") {
            mapCore.set_light(command.handle, command.light);
            return { ok: true };
        }

        post_error("unknown_command", `unknown command type: ${command.type}`, target);
        return { ok: false };
    }
    catch (error) {
        post_error("dispatch", error, target);
        return { ok: false };
    }
}

export function install_bridge(target = default_target()) {
    const bridge = {
        dispatch: (command) => dispatch(command, target),
        map_core: mapCore,
        post: (payload) => post(payload, target),
    };

    if (typeof window !== "undefined" && target === window) {
        window.__gpui_maplibre = bridge;
        window.__gpui_maplibre.dispatch = bridge.dispatch;
        return bridge;
    }

    target[BRIDGE_KEY] = bridge;
    return bridge;
}

const installed_bridge = install_bridge();

if (typeof document !== "undefined") {
    if (document.readyState === "loading") {
        document.addEventListener(
            "DOMContentLoaded",
            () => post_dom_ready(default_target()),
            { once: true },
        );
    }
    else {
        queueMicrotask(() => post_dom_ready(default_target()));
    }
}

export { installed_bridge };
