use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapHandle(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MarkerHandle(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PopupHandle(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ControlHandle(pub u32);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerId(pub String);

impl From<String> for SourceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SourceId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for SourceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<String> for LayerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for LayerId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for LayerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        assert_eq!(serde_json::to_string(&MapHandle(7)).unwrap(), "7");
        assert_eq!(
            serde_json::from_str::<MapHandle>("7").unwrap(),
            MapHandle(7)
        );

        assert_eq!(serde_json::to_string(&MarkerHandle(8)).unwrap(), "8");
        assert_eq!(
            serde_json::from_str::<MarkerHandle>("8").unwrap(),
            MarkerHandle(8)
        );

        assert_eq!(serde_json::to_string(&PopupHandle(9)).unwrap(), "9");
        assert_eq!(
            serde_json::from_str::<PopupHandle>("9").unwrap(),
            PopupHandle(9)
        );

        assert_eq!(serde_json::to_string(&ControlHandle(10)).unwrap(), "10");
        assert_eq!(
            serde_json::from_str::<ControlHandle>("10").unwrap(),
            ControlHandle(10)
        );

        assert_eq!(
            serde_json::to_string(&SourceId::from("places")).unwrap(),
            "\"places\""
        );
        assert_eq!(
            serde_json::from_str::<SourceId>("\"places\"").unwrap(),
            SourceId::from("places")
        );

        assert_eq!(
            serde_json::to_string(&LayerId::from("labels")).unwrap(),
            "\"labels\""
        );
        assert_eq!(
            serde_json::from_str::<LayerId>("\"labels\"").unwrap(),
            LayerId::from("labels")
        );
    }

    #[test]
    fn source_and_layer_ids_are_easy_to_construct() {
        let source = SourceId::from("geojson-source");
        let layer = LayerId::from(String::from("fill-layer"));

        assert_eq!(source.as_ref(), "geojson-source");
        assert_eq!(layer.to_string(), "fill-layer");
    }
}
