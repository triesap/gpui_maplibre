use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, MapLibreError>;

#[derive(Debug)]
pub enum MapLibreError {
    Serialization { source: serde_json::Error },
    InvalidEvent { source: serde_json::Error },
    Transport { message: String },
    Asset { message: String },
    NotReady { context: String },
    MissingHandle { context: String },
    Platform { message: String },
}

impl MapLibreError {
    pub fn invalid_event(source: serde_json::Error) -> Self {
        Self::InvalidEvent { source }
    }

    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    pub fn asset(message: impl Into<String>) -> Self {
        Self::Asset {
            message: message.into(),
        }
    }

    pub fn not_ready(context: impl Into<String>) -> Self {
        Self::NotReady {
            context: context.into(),
        }
    }

    pub fn missing_handle(context: impl Into<String>) -> Self {
        Self::MissingHandle {
            context: context.into(),
        }
    }

    pub fn platform(message: impl Into<String>) -> Self {
        Self::Platform {
            message: message.into(),
        }
    }
}

impl fmt::Display for MapLibreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization { source } => {
                write!(formatter, "failed to serialize MapLibre command: {source}")
            }
            Self::InvalidEvent { source } => {
                write!(formatter, "failed to parse MapLibre event: {source}")
            }
            Self::Transport { message } => {
                write!(formatter, "MapLibre transport failed: {message}")
            }
            Self::Asset { message } => {
                write!(formatter, "MapLibre asset loading failed: {message}")
            }
            Self::NotReady { context } => {
                write!(formatter, "MapLibre runtime is not ready: {context}")
            }
            Self::MissingHandle { context } => {
                write!(formatter, "missing MapLibre handle: {context}")
            }
            Self::Platform { message } => {
                write!(formatter, "MapLibre platform integration failed: {message}")
            }
        }
    }
}

impl Error for MapLibreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialization { source } | Self::InvalidEvent { source } => Some(source),
            Self::Transport { .. }
            | Self::Asset { .. }
            | Self::NotReady { .. }
            | Self::MissingHandle { .. }
            | Self::Platform { .. } => None,
        }
    }
}

impl From<serde_json::Error> for MapLibreError {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialization { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_formats_useful_messages() {
        assert_eq!(
            MapLibreError::transport("webview closed").to_string(),
            "MapLibre transport failed: webview closed"
        );
        assert_eq!(
            MapLibreError::not_ready("waiting for dom_ready").to_string(),
            "MapLibre runtime is not ready: waiting for dom_ready"
        );
        assert_eq!(
            MapLibreError::missing_handle("map was not initialized").to_string(),
            "missing MapLibre handle: map was not initialized"
        );
        assert_eq!(
            MapLibreError::platform("missing WebKitGTK").to_string(),
            "MapLibre platform integration failed: missing WebKitGTK"
        );
        assert_eq!(
            MapLibreError::asset("vendored runtime assets are not enabled").to_string(),
            "MapLibre asset loading failed: vendored runtime assets are not enabled"
        );
    }

    #[test]
    fn serialization_error_keeps_source() {
        let source = serde_json::from_str::<serde_json::Value>("{broken").unwrap_err();
        let error = MapLibreError::from(source);

        assert!(
            error
                .to_string()
                .starts_with("failed to serialize MapLibre command:")
        );
        assert!(error.source().is_some());
    }

    #[test]
    fn invalid_event_error_keeps_source() {
        let source = serde_json::from_str::<serde_json::Value>("{broken").unwrap_err();
        let error = MapLibreError::invalid_event(source);

        assert!(
            error
                .to_string()
                .starts_with("failed to parse MapLibre event:")
        );
        assert!(error.source().is_some());
    }

    #[test]
    fn result_alias_uses_maplibre_error() {
        fn fail() -> Result<()> {
            Err(MapLibreError::not_ready("map initialization pending"))
        }

        let error = fail().unwrap_err();

        assert_eq!(
            error.to_string(),
            "MapLibre runtime is not ready: map initialization pending"
        );
    }
}
