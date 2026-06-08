const INDEX_HTML_TEMPLATE: &str = include_str!("../assets/index.html");
const BRIDGE_JS: &str = include_str!("../assets/bridge.js");
const GPUI_MAPLIBRE_CSS: &str = include_str!("../assets/gpui_maplibre.css");
const MAP_CORE_JS: &str = include_str!("../assets/map_core.js");
#[cfg(feature = "vendored-maplibre")]
const VENDORED_MAPLIBRE_JS: &str = include_str!("../assets/vendor/maplibre-gl.js");
#[cfg(feature = "vendored-maplibre")]
const VENDORED_MAPLIBRE_CSS: &str = include_str!("../assets/vendor/maplibre-gl.css");
const DEFAULT_CDN_VERSION: &str = "5.13.0";
pub const ASSET_PROTOCOL_SCHEME: &str = "gpui-maplibre";
pub const ASSET_PROTOCOL_ORIGIN: &str = "gpui-maplibre://localhost";
pub const ASSET_PROTOCOL_INDEX_PATH: &str = "/index.html";

#[cfg(not(feature = "vendored-maplibre"))]
use crate::MapLibreError;
use crate::{MapInitOptions, Result};
use serde::Serialize;
use std::borrow::Cow;

/// Source for the MapLibre GL JS and CSS runtime used by the WebView.
///
/// Vendored runtime assets remove the MapLibre GL JS/CSS network dependency. Map styles, tiles,
/// glyphs, sprites, and data sources still use the URLs configured in [`MapInitOptions`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapLibreAssets {
    /// Load MapLibre GL JS and CSS from the public unpkg CDN for a specific version.
    Cdn {
        /// MapLibre GL JS package version.
        version: String,
    },
    /// Embed the crate-pinned MapLibre GL JS and CSS files.
    ///
    /// This requires the `vendored-maplibre` Cargo feature. Without that feature,
    /// WebView HTML generation returns a [`MapLibreError`](crate::MapLibreError).
    Vendored,
    /// Embed application-provided MapLibre GL JS and CSS source strings.
    Inline {
        /// JavaScript source for the MapLibre GL runtime.
        js: Cow<'static, str>,
        /// CSS source for the MapLibre GL runtime.
        css: Cow<'static, str>,
    },
    /// Load MapLibre GL JS and CSS from application-provided URLs.
    Urls {
        /// URL for the MapLibre GL JavaScript runtime.
        js_url: String,
        /// URL for the MapLibre GL stylesheet.
        css_url: String,
    },
}

/// Backward-compatible alias for the MapLibre GL runtime asset selector.
pub type AssetMode = MapLibreAssets;

/// Resolved external MapLibre GL JS and CSS URLs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapLibreAssetUrls {
    /// URL for the MapLibre GL JavaScript runtime.
    pub js_url: String,
    /// URL for the MapLibre GL stylesheet.
    pub css_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolAssetResponse {
    pub content_type: &'static str,
    pub body: Cow<'static, [u8]>,
}

enum RuntimeAssets<'a> {
    External {
        js_url: Cow<'a, str>,
        css_url: Cow<'a, str>,
    },
    Inline {
        js: Cow<'a, str>,
        css: Cow<'a, str>,
    },
}

impl Default for MapLibreAssets {
    fn default() -> Self {
        Self::cdn(DEFAULT_CDN_VERSION)
    }
}

impl MapLibreAssets {
    /// Load MapLibre GL JS and CSS from unpkg for `version`.
    pub fn cdn(version: impl Into<String>) -> Self {
        Self::Cdn {
            version: version.into(),
        }
    }

    /// Use the crate-pinned MapLibre GL JS and CSS runtime assets.
    pub fn vendored() -> Self {
        Self::Vendored
    }

    /// Embed application-provided MapLibre GL JS and CSS source strings.
    pub fn inline(js: impl Into<Cow<'static, str>>, css: impl Into<Cow<'static, str>>) -> Self {
        Self::Inline {
            js: js.into(),
            css: css.into(),
        }
    }

    /// Load MapLibre GL JS and CSS from application-provided URLs.
    pub fn urls(js_url: impl Into<String>, css_url: impl Into<String>) -> Self {
        Self::Urls {
            js_url: js_url.into(),
            css_url: css_url.into(),
        }
    }

    /// Load MapLibre GL JS and CSS from application-provided URLs.
    pub fn custom(js_url: impl Into<String>, css_url: impl Into<String>) -> Self {
        Self::urls(js_url, css_url)
    }

    /// Return the external URL representation for CDN and URL-backed assets.
    ///
    /// Inline and vendored assets return stable `inline://` sentinel URLs for compatibility with
    /// low-level callers that inspect the asset mode without rendering inline WebView HTML.
    pub fn maplibre_asset_urls(&self) -> MapLibreAssetUrls {
        match self {
            Self::Cdn { version } => MapLibreAssetUrls {
                js_url: format!("https://unpkg.com/maplibre-gl@{version}/dist/maplibre-gl.js"),
                css_url: format!("https://unpkg.com/maplibre-gl@{version}/dist/maplibre-gl.css"),
            },
            Self::Vendored | Self::Inline { .. } => MapLibreAssetUrls {
                js_url: "inline://maplibre-gl.js".to_owned(),
                css_url: "inline://maplibre-gl.css".to_owned(),
            },
            Self::Urls { js_url, css_url } => MapLibreAssetUrls {
                js_url: js_url.clone(),
                css_url: css_url.clone(),
            },
        }
    }

    fn runtime_assets(&self) -> Result<RuntimeAssets<'_>> {
        match self {
            Self::Cdn { .. } | Self::Urls { .. } => {
                let urls = self.maplibre_asset_urls();
                Ok(RuntimeAssets::External {
                    js_url: Cow::Owned(urls.js_url),
                    css_url: Cow::Owned(urls.css_url),
                })
            }
            Self::Inline { js, css } => Ok(RuntimeAssets::Inline {
                js: Cow::Borrowed(js.as_ref()),
                css: Cow::Borrowed(css.as_ref()),
            }),
            Self::Vendored => vendored_runtime_assets(),
        }
    }
}

pub fn private_index_html(asset_mode: &AssetMode) -> String {
    let urls = asset_mode.maplibre_asset_urls();

    INDEX_HTML_TEMPLATE
        .replace("{{MAPLIBRE_JS_URL}}", &escape_html_attr(&urls.js_url))
        .replace("{{MAPLIBRE_CSS_URL}}", &escape_html_attr(&urls.css_url))
}

pub fn inline_webview_html(asset_mode: &AssetMode, options: &MapInitOptions) -> Result<String> {
    let runtime_assets = asset_mode.runtime_assets()?;
    let map_core_source = script_json(map_core_js())?;
    let bridge_source = script_json(bridge_js())?;
    let init_options = script_json(options)?;
    let maplibre_runtime = maplibre_runtime_html(&runtime_assets);

    Ok(format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script>
      window.__gpui_maplibre_boot_started_at = performance.now();
      window.ipc?.postMessage(JSON.stringify({{
        type: "startup_timing",
        event: {{ milestone: "document_start", elapsed_ms: 0 }}
      }}));
    </script>
    {maplibre_runtime}
    <style>{crate_css}</style>
  </head>
  <body>
    <div id="map"></div>
    <script type="module">
      const mapCoreSource = {map_core_source};
      const bridgeSource = {bridge_source};
      const initOptions = {init_options};
      const mapCoreUrl = URL.createObjectURL(
        new Blob([mapCoreSource], {{ type: "text/javascript" }})
      );
      const bridgeUrl = URL.createObjectURL(
        new Blob([
          bridgeSource.replace(
            'import * as mapCore from "./map_core.js";',
            `import * as mapCore from "${{mapCoreUrl}}";`
          )
        ], {{ type: "text/javascript" }})
      );

      import(bridgeUrl).then((bridge) => {{
        bridge.installed_bridge.dispatch({{ type: "init", options: initOptions }});
      }}).catch((error) => {{
        window.ipc?.postMessage(JSON.stringify({{
          type: "error",
          context: "bootstrap",
          message: error instanceof Error ? error.message : String(error)
        }}));
      }});
    </script>
  </body>
</html>"#,
        maplibre_runtime = maplibre_runtime,
        crate_css = gpui_maplibre_css(),
        map_core_source = map_core_source,
        bridge_source = bridge_source,
        init_options = init_options,
    ))
}

pub fn protocol_webview_url() -> &'static str {
    ASSET_PROTOCOL_ORIGIN
}

pub fn protocol_asset_url(path: &str) -> String {
    format!("{ASSET_PROTOCOL_ORIGIN}{}", normalize_protocol_path(path))
}

pub fn protocol_index_html(asset_mode: &AssetMode, options: &MapInitOptions) -> Result<String> {
    let runtime_assets = protocol_runtime_assets(asset_mode)?;
    let init_options = script_json(options)?;
    let bridge_url = protocol_asset_url("/bridge.js");

    Ok(format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script>
      window.__gpui_maplibre_boot_started_at = performance.now();
      window.ipc?.postMessage(JSON.stringify({{
        type: "startup_timing",
        event: {{ milestone: "document_start", elapsed_ms: 0 }}
      }}));
    </script>
    {runtime_assets}
    <link rel="stylesheet" href="{crate_css_url}">
  </head>
  <body>
    <div id="map"></div>
    <script type="module">
      import {{ installed_bridge }} from "{bridge_url}";
      const initOptions = {init_options};

      installed_bridge.dispatch({{ type: "init", options: initOptions }});
    </script>
  </body>
</html>"#,
        runtime_assets = runtime_assets,
        crate_css_url = protocol_asset_url("/gpui_maplibre.css"),
        bridge_url = escape_html_attr(&bridge_url),
        init_options = init_options,
    ))
}

pub fn protocol_asset_response(
    path: &str,
    asset_mode: &AssetMode,
    options: &MapInitOptions,
) -> Result<Option<ProtocolAssetResponse>> {
    let path = normalize_protocol_path(path);
    let response = match path.as_str() {
        "/" | "/index.html" => ProtocolAssetResponse {
            content_type: "text/html; charset=utf-8",
            body: Cow::Owned(protocol_index_html(asset_mode, options)?.into_bytes()),
        },
        "/bridge.js" => ProtocolAssetResponse {
            content_type: "text/javascript; charset=utf-8",
            body: Cow::Borrowed(bridge_js().as_bytes()),
        },
        "/map_core.js" => ProtocolAssetResponse {
            content_type: "text/javascript; charset=utf-8",
            body: Cow::Borrowed(map_core_js().as_bytes()),
        },
        "/gpui_maplibre.css" => ProtocolAssetResponse {
            content_type: "text/css; charset=utf-8",
            body: Cow::Borrowed(gpui_maplibre_css().as_bytes()),
        },
        "/vendor/maplibre-gl.js" => vendored_protocol_asset_response(
            "text/javascript; charset=utf-8",
            VendoredProtocolAsset::Js,
        )?,
        "/vendor/maplibre-gl.css" => {
            vendored_protocol_asset_response("text/css; charset=utf-8", VendoredProtocolAsset::Css)?
        }
        _ => return Ok(None),
    };

    Ok(Some(response))
}

#[cfg(feature = "vendored-maplibre")]
fn vendored_runtime_assets() -> Result<RuntimeAssets<'static>> {
    Ok(RuntimeAssets::Inline {
        js: Cow::Borrowed(VENDORED_MAPLIBRE_JS),
        css: Cow::Borrowed(VENDORED_MAPLIBRE_CSS),
    })
}

#[cfg(not(feature = "vendored-maplibre"))]
fn vendored_runtime_assets() -> Result<RuntimeAssets<'static>> {
    Err(MapLibreError::asset(
        "vendored MapLibre GL assets require the vendored-maplibre feature",
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VendoredProtocolAsset {
    Js,
    Css,
}

#[cfg(feature = "vendored-maplibre")]
fn vendored_protocol_asset_response(
    content_type: &'static str,
    asset: VendoredProtocolAsset,
) -> Result<ProtocolAssetResponse> {
    let body = match asset {
        VendoredProtocolAsset::Js => VENDORED_MAPLIBRE_JS.as_bytes(),
        VendoredProtocolAsset::Css => VENDORED_MAPLIBRE_CSS.as_bytes(),
    };

    Ok(ProtocolAssetResponse {
        content_type,
        body: Cow::Borrowed(body),
    })
}

#[cfg(not(feature = "vendored-maplibre"))]
fn vendored_protocol_asset_response(
    _: &'static str,
    _: VendoredProtocolAsset,
) -> Result<ProtocolAssetResponse> {
    Err(MapLibreError::asset(
        "vendored MapLibre GL assets require the vendored-maplibre feature",
    ))
}

fn protocol_runtime_assets(asset_mode: &AssetMode) -> Result<String> {
    match asset_mode {
        AssetMode::Vendored => Ok(format!(
            r#"<link rel="stylesheet" href="{css_url}">
    <script src="{js_url}"></script>"#,
            css_url = protocol_asset_url("/vendor/maplibre-gl.css"),
            js_url = protocol_asset_url("/vendor/maplibre-gl.js"),
        )),
        _ => Ok(maplibre_runtime_html(&asset_mode.runtime_assets()?)),
    }
}

fn maplibre_runtime_html(runtime_assets: &RuntimeAssets<'_>) -> String {
    match runtime_assets {
        RuntimeAssets::External { js_url, css_url } => format!(
            r#"<link rel="stylesheet" href="{css_url}">
    <script src="{js_url}"></script>"#,
            css_url = escape_html_attr(css_url),
            js_url = escape_html_attr(js_url),
        ),
        RuntimeAssets::Inline { js, css } => format!(
            r#"<style data-gpui-maplibre-runtime-css>{css}</style>
    <script data-gpui-maplibre-runtime-js>{js}</script>"#
        ),
    }
}

fn normalize_protocol_path(path: &str) -> String {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if path.is_empty() || path == "/" {
        return ASSET_PROTOCOL_INDEX_PATH.to_owned();
    }
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

pub fn gpui_maplibre_css() -> &'static str {
    GPUI_MAPLIBRE_CSS
}

pub fn bridge_js() -> &'static str {
    BRIDGE_JS
}

pub fn map_core_js() -> &'static str {
    MAP_CORE_JS
}

fn escape_html_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn script_json(value: impl Serialize) -> Result<String> {
    Ok(serde_json::to_string(&value)?.replace("</", "<\\/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_matches(haystack: &str, needle: &str) -> usize {
        haystack.match_indices(needle).count()
    }

    #[test]
    fn asset_html_contains_single_map_container_and_private_assets() {
        let html = private_index_html(&AssetMode::default());

        assert_eq!(count_matches(&html, r#"id="map""#), 1);
        assert!(html.contains(r#"href="./gpui_maplibre.css""#));
        assert!(html.contains(r#"src="./bridge.js""#));
        assert!(html.contains("maplibre-gl@5.13.0"));
        assert!(!html.to_lowercase().contains("trunk"));
        assert!(!html.to_lowercase().contains("wasm"));
    }

    #[test]
    fn asset_html_supports_cdn_and_custom_url_modes() {
        let cdn = AssetMode::cdn("5.14.0").maplibre_asset_urls();
        assert_eq!(
            cdn.js_url,
            "https://unpkg.com/maplibre-gl@5.14.0/dist/maplibre-gl.js"
        );
        assert_eq!(
            cdn.css_url,
            "https://unpkg.com/maplibre-gl@5.14.0/dist/maplibre-gl.css"
        );

        let custom = private_index_html(&AssetMode::custom(
            "app://assets/maplibre.js",
            "app://assets/maplibre.css",
        ));
        assert!(custom.contains(r#"src="app://assets/maplibre.js""#));
        assert!(custom.contains(r#"href="app://assets/maplibre.css""#));
    }

    #[test]
    fn asset_html_escapes_custom_urls() {
        let html = private_index_html(&AssetMode::custom(
            r#"app://assets/maplibre.js?name="quoted""#,
            "app://assets/maplibre.css?a=1&b=2",
        ));

        assert!(html.contains("name=&quot;quoted&quot;"));
        assert!(html.contains("a=1&amp;b=2"));
    }

    #[test]
    fn asset_html_css_defines_full_size_map_surface() {
        let css = gpui_maplibre_css();

        assert!(css.contains("#map"));
        assert!(css.contains("height: 100%"));
        assert!(css.contains("overflow: hidden"));
    }

    #[test]
    fn inline_webview_html_embeds_private_bridge_assets() {
        let html = inline_webview_html(&AssetMode::default(), &MapInitOptions::default()).unwrap();

        assert!(html.contains("const mapCoreSource = "));
        assert!(html.contains("const bridgeSource = "));
        assert!(html.contains("bridge.installed_bridge.dispatch"));
        assert!(html.contains("type: \"init\""));
        assert!(html.contains("maplibre-gl@5.13.0"));
        assert!(!html.contains(r#"src="./bridge.js""#));
        assert!(!html.contains(r#"href="./gpui_maplibre.css""#));
    }

    #[cfg(not(feature = "vendored-maplibre"))]
    #[test]
    fn maplibre_assets_exposes_vendored_runtime_api() {
        let vendored = MapLibreAssets::vendored();
        let error = inline_webview_html(&vendored, &MapInitOptions::default()).unwrap_err();

        assert_eq!(
            error.to_string(),
            "MapLibre asset loading failed: vendored MapLibre GL assets require the vendored-maplibre feature"
        );
    }

    #[cfg(feature = "vendored-maplibre")]
    #[test]
    fn vendored_runtime_assets_embed_pinned_maplibre_runtime() {
        let html =
            inline_webview_html(&MapLibreAssets::vendored(), &MapInitOptions::default()).unwrap();

        assert!(html.contains("data-gpui-maplibre-runtime-js"));
        assert!(html.contains("data-gpui-maplibre-runtime-css"));
        assert!(html.contains("maplibregl"));
        assert!(!html.contains("https://unpkg.com"));
        assert!(!html.contains(r#"<script src="inline://maplibre-gl.js""#));
    }

    #[test]
    fn inline_webview_html_supports_app_owned_runtime_assets() {
        let assets = MapLibreAssets::inline(
            "globalThis.maplibregl = { Map: function() {} };",
            ".maplibregl-map { position: relative; }",
        );
        let html = inline_webview_html(&assets, &MapInitOptions::default()).unwrap();

        assert!(html.contains("data-gpui-maplibre-runtime-js"));
        assert!(html.contains("data-gpui-maplibre-runtime-css"));
        assert!(html.contains("globalThis.maplibregl"));
        assert!(!html.contains("https://unpkg.com"));
    }

    #[test]
    fn protocol_asset_response_serves_crate_runtime_files() {
        let options = MapInitOptions::default();
        let html = protocol_asset_response("/index.html", &AssetMode::default(), &options)
            .unwrap()
            .expect("index response");
        let bridge = protocol_asset_response("/bridge.js", &AssetMode::default(), &options)
            .unwrap()
            .expect("bridge response");
        let map_core = protocol_asset_response("/map_core.js", &AssetMode::default(), &options)
            .unwrap()
            .expect("map core response");
        let css = protocol_asset_response("/gpui_maplibre.css", &AssetMode::default(), &options)
            .unwrap()
            .expect("crate css response");

        assert_eq!(html.content_type, "text/html; charset=utf-8");
        assert_eq!(bridge.content_type, "text/javascript; charset=utf-8");
        assert_eq!(map_core.content_type, "text/javascript; charset=utf-8");
        assert_eq!(css.content_type, "text/css; charset=utf-8");
        assert!(String::from_utf8_lossy(&html.body).contains("installed_bridge.dispatch"));
        assert!(String::from_utf8_lossy(&bridge.body).contains("window.__gpui_maplibre.dispatch"));
        assert!(String::from_utf8_lossy(&map_core.body).contains("export function init_map"));
        assert!(String::from_utf8_lossy(&css.body).contains("#map"));
    }

    #[cfg(feature = "vendored-maplibre")]
    #[test]
    fn vendored_protocol_index_references_runtime_urls() {
        let options = MapInitOptions::default();
        let response =
            protocol_asset_response("/index.html", &MapLibreAssets::vendored(), &options)
                .unwrap()
                .expect("index response");
        let html = String::from_utf8(response.body.into_owned()).unwrap();

        assert!(html.contains(r#"src="gpui-maplibre://localhost/vendor/maplibre-gl.js""#));
        assert!(html.contains(r#"href="gpui-maplibre://localhost/vendor/maplibre-gl.css""#));
        assert!(html.contains(r#"href="gpui-maplibre://localhost/gpui_maplibre.css""#));
        assert!(!html.contains("data-gpui-maplibre-runtime-js"));
        assert!(!html.contains("maplibregl.LngLat"));
    }

    #[cfg(feature = "vendored-maplibre")]
    #[test]
    fn vendored_protocol_index_avoids_inline_runtime_payload() {
        let options = MapInitOptions::default();
        let protocol_html = protocol_index_html(&MapLibreAssets::vendored(), &options).unwrap();
        let inline_html =
            inline_webview_html(&MapLibreAssets::vendored(), &options).expect("inline html");

        assert!(inline_html.len() > protocol_html.len() * 100);
        assert!(inline_html.contains("maplibregl"));
        assert!(!protocol_html.contains("maplibregl"));
        assert!(protocol_html.contains("vendor/maplibre-gl.js"));
        assert!(protocol_html.contains("vendor/maplibre-gl.css"));
    }

    #[cfg(feature = "vendored-maplibre")]
    #[test]
    fn vendored_protocol_asset_response_serves_pinned_runtime() {
        let options = MapInitOptions::default();
        let js = protocol_asset_response(
            "/vendor/maplibre-gl.js",
            &MapLibreAssets::vendored(),
            &options,
        )
        .unwrap()
        .expect("vendored js");
        let css = protocol_asset_response(
            "/vendor/maplibre-gl.css",
            &MapLibreAssets::vendored(),
            &options,
        )
        .unwrap()
        .expect("vendored css");

        assert_eq!(js.content_type, "text/javascript; charset=utf-8");
        assert_eq!(css.content_type, "text/css; charset=utf-8");
        assert!(String::from_utf8_lossy(&js.body).contains("maplibregl"));
        assert!(String::from_utf8_lossy(&css.body).contains("maplibregl"));
    }

    #[test]
    fn inline_webview_html_escapes_script_breaking_options() {
        let options = MapInitOptions::default()
            .with_style_url(r#"https://example.test/style.json?</script><script>"#);
        let html = inline_webview_html(&AssetMode::default(), &options).unwrap();

        assert!(html.contains(r#"<\/script><script>"#));
        assert!(!html.contains(r#"?</script><script>"#));
    }

    #[test]
    fn map_core_asset_exports_reference_bridge_functions() {
        let js = map_core_js();

        for export_name in [
            "init_map",
            "destroy_map",
            "fly_to",
            "add_source",
            "add_layer",
            "create_marker",
            "create_popup",
            "register_on_map_events",
            "register_on_layer_events",
        ] {
            assert!(
                js.contains(&format!("export function {export_name}(")),
                "missing map_core export {export_name}"
            );
        }
    }

    #[test]
    fn map_core_asset_mode_contract_supports_configured_maplibre_loading() {
        let js = map_core_js();
        let cdn_html = private_index_html(&AssetMode::cdn("5.13.0"));
        let local_html = private_index_html(&AssetMode::urls(
            "./vendor/maplibre-gl.js",
            "./vendor/maplibre-gl.css",
        ));

        assert!(!js.contains("https://esm.sh/maplibre-gl"));
        assert!(js.contains("export function configure_maplibre_gl"));
        assert!(js.contains("export async function load_maplibre_gl"));
        assert!(js.contains("globalThis.maplibregl"));
        assert!(js.contains("await import(module_url)"));
        assert!(cdn_html.contains("https://unpkg.com/maplibre-gl@5.13.0"));
        assert!(local_html.contains("./vendor/maplibre-gl.js"));
        assert!(local_html.contains("./vendor/maplibre-gl.css"));
    }

    #[test]
    fn bridge_asset_contract_defines_dispatcher_ipc_and_dom_ready() {
        let js = bridge_js();

        assert!(js.contains(r#"import * as mapCore from "./map_core.js""#));
        assert!(js.contains("window.__gpui_maplibre.dispatch"));
        assert!(js.contains("window.ipc.postMessage"));
        assert!(js.contains(r#"type: "dom_ready""#));
        assert!(js.contains("unknown_command"));
        assert!(js.contains(r#"type: "error""#));
    }
}
