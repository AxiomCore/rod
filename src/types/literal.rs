use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodLiteral {
    expected: Value,
}

impl RodLiteral {
    pub fn new(expected: Value) -> Self {
        Self { expected }
    }
}

impl RodValidator for RodLiteral {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // Optimization: Compare input against expected primitives without allocating full JSON
        let is_match = match &self.expected {
            Value::String(s) => {
                if let Some(input_str) = input.as_str() {
                    input_str.as_ref() == s
                } else {
                    false
                }
            }
            Value::Number(n) => {
                if let Some(input_num) = input.as_f64() {
                    if let Some(expected_f64) = n.as_f64() {
                        // Direct float comparison (acceptable for literals usually)
                        // Use a tiny epsilon if strict equality fails?
                        // For now strict equality matches serde behavior.
                        (input_num - expected_f64).abs() < f64::EPSILON
                    } else {
                        false // Expected isn't a simple f64 (e.g. arbitrary precision)
                    }
                } else {
                    false
                }
            }
            Value::Bool(b) => {
                if let Some(input_bool) = input.as_bool() {
                    input_bool == *b
                } else {
                    false
                }
            }
            Value::Null => input.get_type() == crate::core::input::DataType::Null,
            // For complex types (Arrays/Objects), fall back to slow serialization
            _ => {
                let val = input.to_json();
                val == self.expected
            }
        };

        if is_match {
            // Success: Return Zero-Copy value
            // We reconstruct the output based on what we know the type is.
            match &self.expected {
                Value::String(_) => {
                    let s = input.as_str().unwrap(); // Safe because we checked above
                    return Ok(RodValue::String(s));
                }
                Value::Number(_) => {
                    let n = input.as_f64().unwrap();
                    return Ok(RodValue::Number(n));
                }
                Value::Bool(b) => return Ok(RodValue::Boolean(*b)),
                Value::Null => return Ok(RodValue::Null),
                _ => {
                    // Fallback was used, return the json we created or create new
                    // Since we didn't save the 'val' from fallback check above to avoid clone in match arm,
                    // we might need to recreate it or just return a generic success.
                    // Actually, for complex types, let's just use to_json for return too.
                    return Ok(RodValue::Json(input.to_json()));
                }
            }
        }

        Err(RodError::new(
            "invalid_literal",
            &format!("Invalid literal value, expected {}", self.expected),
        ))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn literal<T: Into<Value>>(val: T) -> RodLiteral {
    RodLiteral::new(val.into())
}
