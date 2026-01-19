// src/lib.rs
pub mod core;
pub mod error;
pub mod io;
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

// Export IO
pub use core::input::RodInput;
pub use core::validator::RodValidator;
pub use schema::parser::from_yaml;

// Coercion Helpers (z.coerce)
pub mod coerce {
    use super::*;
    use crate::core::input::{DataType, RodInput};
    use serde_json::{Value, json};

    pub fn string() -> impl RodValidator {
        preprocess(
            |v: &dyn RodInput| {
                // Check type using trait methods
                // Preprocess must return an OWNED Value, which is then wrapped by RodPreprocess logic
                match v.get_type() {
                    DataType::String => Value::String(v.as_str().unwrap().to_string()),
                    DataType::Number => Value::String(v.as_f64().unwrap().to_string()),
                    DataType::Boolean => Value::String(v.as_bool().unwrap().to_string()),
                    // Fallback: convert whatever input we have to JSON value to pass through
                    _ => v.to_json(),
                }
            },
            crate::types::string::string(),
        )
    }

    pub fn number() -> impl RodValidator {
        preprocess(
            |v: &dyn RodInput| {
                match v.get_type() {
                    DataType::Number => {
                        // as_f64 is safe because we checked type
                        let f = v.as_f64().unwrap();
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            Value::Number(n)
                        } else {
                            // NaN/Inf logic? Zod coerce returns NaN for strings, but JSON doesn't support it.
                            // Fallback to null or original
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
                        // Failed to parse, pass original string to let validator fail
                        Value::String(s.to_string())
                    }
                    _ => v.to_json(),
                }
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
