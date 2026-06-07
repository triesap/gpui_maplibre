use crate::{MapCommand, Result};

#[allow(dead_code)]
const DISPATCH_PREFIX: &str = "window.__gpui_maplibre.dispatch(JSON.parse(";
#[allow(dead_code)]
const DISPATCH_SUFFIX: &str = "));";

#[allow(dead_code)]
pub(crate) fn script_for_command(command: &MapCommand) -> Result<String> {
    let json = serde_json::to_string(command)?;
    let js_string_literal = serde_json::to_string(&json)?;

    Ok(format!(
        "{DISPATCH_PREFIX}{js_string_literal}{DISPATCH_SUFFIX}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MapHandle, PopupContent, PopupOptions};
    use serde_json::json;

    fn command_json_from_script(script: &str) -> String {
        let literal = script
            .strip_prefix(DISPATCH_PREFIX)
            .and_then(|value| value.strip_suffix(DISPATCH_SUFFIX))
            .expect("script uses dispatcher wrapper");

        serde_json::from_str::<String>(literal).expect("script contains a JSON string literal")
    }

    #[test]
    fn script_for_command_wraps_dispatcher_call() {
        let command = MapCommand::Resize {
            handle: MapHandle(1),
        };

        let script = script_for_command(&command).unwrap();
        let command_json = command_json_from_script(&script);

        assert!(script.starts_with(DISPATCH_PREFIX));
        assert!(script.ends_with(DISPATCH_SUFFIX));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&command_json).unwrap(),
            json!({
                "type": "resize",
                "handle": 1
            })
        );
    }

    #[test]
    fn script_for_command_escapes_quotes_and_script_tags() {
        let hostile = "quote\" backslash\\ newline\n unicode λ </script> ); window.evil = true; //";
        let command = MapCommand::SetStyle {
            handle: MapHandle(1),
            style_url: hostile.to_owned(),
        };

        let script = script_for_command(&command).unwrap();
        let command_json = command_json_from_script(&script);
        let decoded = serde_json::from_str::<serde_json::Value>(&command_json).unwrap();

        assert!(!script.contains('\n'));
        assert!(!script.contains("JSON.parse({\"type\""));
        assert_eq!(decoded["style_url"], hostile);
    }

    #[test]
    fn script_for_command_preserves_popup_content_safely() {
        let command = MapCommand::CreatePopup {
            request_id: 7,
            handle: MapHandle(1),
            options: PopupOptions {
                lng: -123.1,
                lat: 49.2,
                content: PopupContent::Text("<b>not html</b>".to_owned()),
                close_button: Some(true),
                close_on_click: Some(false),
                anchor: None,
                offset_x: None,
                offset_y: None,
                max_width: None,
            },
        };

        let script = script_for_command(&command).unwrap();
        let command_json = command_json_from_script(&script);
        let decoded = serde_json::from_str::<serde_json::Value>(&command_json).unwrap();

        assert_eq!(decoded["type"], "create_popup");
        assert_eq!(decoded["options"]["content"]["kind"], "text");
        assert_eq!(decoded["options"]["content"]["value"], "<b>not html</b>");
    }
}
