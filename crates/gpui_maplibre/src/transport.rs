use crate::{MapCommand, MapLibreError, Result};

pub trait CommandTransport {
    fn send_command(&mut self, command: MapCommand) -> Result<()>;
}

#[cfg(feature = "gpui-webview")]
pub struct GpuiWebViewTransport<'a> {
    webview: &'a gpui_wry::WebView,
}

#[cfg(feature = "gpui-webview")]
impl<'a> GpuiWebViewTransport<'a> {
    pub fn new(webview: &'a gpui_wry::WebView) -> Self {
        Self { webview }
    }

    pub fn webview(&self) -> &'a gpui_wry::WebView {
        self.webview
    }
}

#[cfg(feature = "gpui-webview")]
impl CommandTransport for GpuiWebViewTransport<'_> {
    fn send_command(&mut self, command: MapCommand) -> Result<()> {
        let script = crate::script::script_for_command(&command)?;
        self.webview
            .evaluate_script(&script)
            .map_err(|error| MapLibreError::platform(error.to_string()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct FakeTransport {
    commands: Vec<MapCommand>,
    next_error: Option<String>,
}

impl FakeTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn commands(&self) -> &[MapCommand] {
        &self.commands
    }

    pub fn into_commands(self) -> Vec<MapCommand> {
        self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn fail_next(&mut self, message: impl Into<String>) {
        self.next_error = Some(message.into());
    }
}

impl CommandTransport for FakeTransport {
    fn send_command(&mut self, command: MapCommand) -> Result<()> {
        if let Some(message) = self.next_error.take() {
            return Err(MapLibreError::transport(message));
        }

        self.commands.push(command);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MapHandle;

    #[test]
    fn transport_fake_records_commands_in_order() {
        let mut transport = FakeTransport::new();

        transport
            .send_command(MapCommand::Resize {
                handle: MapHandle(1),
            })
            .unwrap();
        transport
            .send_command(MapCommand::Destroy {
                handle: MapHandle(1),
            })
            .unwrap();

        assert_eq!(
            transport.commands(),
            &[
                MapCommand::Resize {
                    handle: MapHandle(1),
                },
                MapCommand::Destroy {
                    handle: MapHandle(1),
                }
            ]
        );
    }

    #[test]
    fn transport_fake_can_fail_next_command() {
        let mut transport = FakeTransport::new();

        transport.fail_next("bridge closed");
        let error = transport
            .send_command(MapCommand::Resize {
                handle: MapHandle(1),
            })
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "MapLibre transport failed: bridge closed"
        );
        assert!(transport.commands().is_empty());

        transport
            .send_command(MapCommand::Destroy {
                handle: MapHandle(1),
            })
            .unwrap();
        assert_eq!(
            transport.commands(),
            &[MapCommand::Destroy {
                handle: MapHandle(1),
            }]
        );
    }
}
