use jsonschema::{JSONSchema, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, derive_more::AsRef)]
pub struct JSONSchemaShim {
    compiled: JSONSchema,
    raw: serde_json::Value,
}

impl JSONSchemaShim {
    pub fn new(schema: &serde_json::Value) -> Result<Self, ValidationError> {
        let raw = schema.clone();
        let compiled = JSONSchema::compile(&schema)?;
        Ok(Self { compiled, raw })
    }
}

impl<'de> Deserialize<'de> for JSONSchemaShim {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::new(&raw).map_err(serde::de::Error::custom)?)
    }
}

impl Serialize for JSONSchemaShim {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.raw.serialize(serializer)
    }
}

impl Clone for JSONSchemaShim {
    fn clone(&self) -> Self {
        Self::new(&self.raw).unwrap()
    }
}
