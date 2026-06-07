const INDEX_HTML_TEMPLATE: &str = include_str!("../assets/index.html");
const GPUI_MAPLIBRE_CSS: &str = include_str!("../assets/gpui_maplibre.css");
const DEFAULT_CDN_VERSION: &str = "5.13.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetMode {
    Cdn { version: String },
    VendoredPlaceholder,
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

pub fn gpui_maplibre_css() -> &'static str {
    GPUI_MAPLIBRE_CSS
}

fn escape_html_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
}
