const INDEX_HTML_TEMPLATE: &str = include_str!("../assets/index.html");
const BRIDGE_JS: &str = include_str!("../assets/bridge.js");
const GPUI_MAPLIBRE_CSS: &str = include_str!("../assets/gpui_maplibre.css");
const MAP_CORE_JS: &str = include_str!("../assets/map_core.js");
const DEFAULT_CDN_VERSION: &str = "5.13.0";

use crate::{MapInitOptions, Result};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetMode {
    /// Load MapLibre GL JS and CSS from CDN script/link tags in the private HTML.
    Cdn { version: String },
    /// Reserve relative vendored URLs for future local packaging without network access.
    VendoredPlaceholder,
    /// Load caller-provided MapLibre GL JS and CSS URLs in the private HTML.
    Custom { js_url: String, css_url: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapLibreAssetUrls {
    pub js_url: String,
    pub css_url: String,
}

impl Default for AssetMode {
    fn default() -> Self {
        Self::cdn(DEFAULT_CDN_VERSION)
    }
}

impl AssetMode {
    pub fn cdn(version: impl Into<String>) -> Self {
        Self::Cdn {
            version: version.into(),
        }
    }

    pub fn custom(js_url: impl Into<String>, css_url: impl Into<String>) -> Self {
        Self::Custom {
            js_url: js_url.into(),
            css_url: css_url.into(),
        }
    }

    pub fn maplibre_asset_urls(&self) -> MapLibreAssetUrls {
        match self {
            Self::Cdn { version } => MapLibreAssetUrls {
                js_url: format!("https://unpkg.com/maplibre-gl@{version}/dist/maplibre-gl.js"),
                css_url: format!("https://unpkg.com/maplibre-gl@{version}/dist/maplibre-gl.css"),
            },
            Self::VendoredPlaceholder => MapLibreAssetUrls {
                js_url: "./vendor/maplibre-gl.js".to_owned(),
                css_url: "./vendor/maplibre-gl.css".to_owned(),
            },
            Self::Custom { js_url, css_url } => MapLibreAssetUrls {
                js_url: js_url.clone(),
                css_url: css_url.clone(),
            },
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
    let urls = asset_mode.maplibre_asset_urls();
    let map_core_source = script_json(map_core_js())?;
    let bridge_source = script_json(bridge_js())?;
    let init_options = script_json(options)?;

    Ok(format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" href="{css_url}">
    <style>{crate_css}</style>
    <script src="{js_url}"></script>
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
        css_url = escape_html_attr(&urls.css_url),
        js_url = escape_html_attr(&urls.js_url),
        crate_css = gpui_maplibre_css(),
        map_core_source = map_core_source,
        bridge_source = bridge_source,
        init_options = init_options,
    ))
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
    fn asset_html_supports_cdn_vendored_and_custom_modes() {
        let cdn = AssetMode::cdn("5.14.0").maplibre_asset_urls();
        assert_eq!(
            cdn.js_url,
            "https://unpkg.com/maplibre-gl@5.14.0/dist/maplibre-gl.js"
        );
        assert_eq!(
            cdn.css_url,
            "https://unpkg.com/maplibre-gl@5.14.0/dist/maplibre-gl.css"
        );

        let vendored = AssetMode::VendoredPlaceholder.maplibre_asset_urls();
        assert_eq!(vendored.js_url, "./vendor/maplibre-gl.js");
        assert_eq!(vendored.css_url, "./vendor/maplibre-gl.css");

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
        let vendored_html = private_index_html(&AssetMode::VendoredPlaceholder);

        assert!(!js.contains("https://esm.sh/maplibre-gl"));
        assert!(js.contains("export function configure_maplibre_gl"));
        assert!(js.contains("export async function load_maplibre_gl"));
        assert!(js.contains("globalThis.maplibregl"));
        assert!(js.contains("await import(module_url)"));
        assert!(cdn_html.contains("https://unpkg.com/maplibre-gl@5.13.0"));
        assert!(vendored_html.contains("./vendor/maplibre-gl.js"));
        assert!(vendored_html.contains("./vendor/maplibre-gl.css"));
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
