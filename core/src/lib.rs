pub mod core;
pub mod error;
pub mod io;
pub mod macros;
pub mod schema;
pub mod types;

use serde_json::Value;

// Export types factories
pub use types::array::array;
pub use types::boolean::boolean;
pub use types::date::date;
pub use types::discriminated_union::discriminated_union;
pub use types::effect::{refine, transform};
pub use types::enum_type::enum_type;
pub use types::intersection::intersection;
pub use types::lazy::lazy;
pub use types::literal::literal;
pub use types::map::map;
pub use types::node::{IntoRodNode, RodNode}; // Added node exports
pub use types::nullable::NullableExtension;
pub use types::number::number;
pub use types::object::object;
pub use types::optional::OptionalExtension;
pub use types::preprocess::preprocess;
pub use types::primitive::{any, never};
pub use types::record::record;
pub use types::set::set;
pub use types::string::string;
pub use types::tuple::tuple;
pub use types::union::union;

// Export IO & Core Traits
pub use core::async_validator::{AsyncRodValidator, BoxFuture};
pub use core::input::RodInput;
pub use core::validator::RodValidator;
pub use core::value::RodValue;
pub use schema::parser::from_yaml;

/// Parse a Rod schema from YAML and return the optimized RodNode.
pub fn from_yaml_node(content: &str) -> Result<RodNode, serde_yaml::Error> {
    let spec: schema::parser::RodSpec = serde_yaml::from_str(content)?;
    Ok(spec.build_node())
}

// Coercion Helpers (z.coerce)
pub mod coerce {
    use super::*;
    use crate::core::input::{DataType, RodInput};
    use serde_json::{Value, json};

    pub fn string() -> RodNode {
        preprocess(
            |v: &dyn RodInput| match v.get_type() {
                DataType::String => Value::String(v.as_str().unwrap().to_string()),
                DataType::Number => Value::String(v.as_f64().unwrap().to_string()),
                DataType::Boolean => Value::String(v.as_bool().unwrap().to_string()),
                _ => v.to_json(),
            },
            crate::types::string::string(),
        )
    }

    pub fn number() -> RodNode {
        preprocess(
            |v: &dyn RodInput| match v.get_type() {
                DataType::Number => {
                    let f = v.as_f64().unwrap();
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        Value::Number(n)
                    } else {
                        Value::Null
                    }
                }
                DataType::String => {
                    let s = v.as_str().unwrap();
                    if let Ok(i) = s.parse::<i64>() {
                        return json!(i);
                    }
                    if let Ok(f) = s.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            return Value::Number(n);
                        }
                    }
                    Value::String(s.to_string())
                }
                _ => v.to_json(),
            },
            crate::types::number::number(),
        )
    }

    pub fn boolean() -> RodNode {
        preprocess(
            |v: &dyn RodInput| match v.get_type() {
                DataType::String => {
                    let s = v.as_str().unwrap();
                    Value::Bool(s == "true")
                }
                DataType::Number => {
                    let f = v.as_f64().unwrap();
                    Value::Bool(f != 0.0)
                }
                _ => v.to_json(),
            },
            crate::types::boolean::boolean(),
        )
    }
}

// Helper for type name inspection
pub fn get_type_name(v: &Value) -> String {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
    .to_string()
}

// Deserialization helper
use serde::de::DeserializeOwned;
impl<'a> crate::core::value::RodValue<'a> {
    pub fn parse_into<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        T::deserialize(self)
    }
}
