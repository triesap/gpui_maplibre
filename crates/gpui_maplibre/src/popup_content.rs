use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PopupContent {
    Text(String),
    TrustedHtml(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn popup_content_text_and_trusted_html_are_distinct() {
        assert_eq!(
            serde_json::to_value(PopupContent::Text("<b>plain text</b>".to_owned())).unwrap(),
            json!({
                "kind": "text",
                "value": "<b>plain text</b>"
            })
        );

        assert_eq!(
            serde_json::to_value(PopupContent::TrustedHtml(
                "<strong>trusted markup</strong>".to_owned()
            ))
            .unwrap(),
            json!({
                "kind": "trusted_html",
                "value": "<strong>trusted markup</strong>"
            })
        );

        assert_ne!(
            PopupContent::Text("<em>value</em>".to_owned()),
            PopupContent::TrustedHtml("<em>value</em>".to_owned())
        );
    }

    #[test]
    fn popup_content_roundtrips_from_wire_format() {
        let text = serde_json::from_value::<PopupContent>(json!({
            "kind": "text",
            "value": "safe text"
        }))
        .unwrap();
        let trusted_html = serde_json::from_value::<PopupContent>(json!({
            "kind": "trusted_html",
            "value": "<span>reviewed</span>"
        }))
        .unwrap();

        assert_eq!(text, PopupContent::Text("safe text".to_owned()));
        assert_eq!(
            trusted_html,
            PopupContent::TrustedHtml("<span>reviewed</span>".to_owned())
        );
    }
}
