// src/lib.rs
pub mod core;
pub mod error;
pub mod macros;
pub mod schema;
pub mod types;

use serde_json::Value;
// Export types
pub use types::array::array;
pub use types::boolean::boolean;
pub use types::date::date;
pub use types::discriminated_union::{discriminated_union, discriminated_union_map};
pub use types::effect::{refine, transform};
pub use types::enum_type::enum_type;
pub use types::intersection::intersection;
pub use types::lazy::lazy;
pub use types::literal::literal;
pub use types::map::map;
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

pub use core::validator::RodValidator;
pub use schema::parser::from_yaml;

// Coercion Helpers (z.coerce)
pub mod coerce {
    use super::*;
    use serde_json::{Value, json};

    pub fn string() -> impl RodValidator {
        preprocess(
            |v| match v {
                Value::String(s) => Value::String(s.clone()),
                Value::Number(n) => Value::String(n.to_string()),
                Value::Bool(b) => Value::String(b.to_string()),
                _ => v.clone(),
            },
            crate::types::string::string(),
        )
    }

    pub fn number() -> impl RodValidator {
        preprocess(
            |v| match v {
                Value::Number(n) => Value::Number(n.clone()),
                Value::String(s) => {
                    if let Ok(i) = s.parse::<i64>() {
                        return json!(i);
                    }
                    if let Ok(f) = s.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            return Value::Number(n);
                        }
                    }
                    v.clone()
                }
                _ => v.clone(),
            },
            crate::types::number::number(),
        )
    }
}
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
