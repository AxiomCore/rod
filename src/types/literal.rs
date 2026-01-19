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
        // Compare input.to_json() with expected
        // NOTE: This triggers a conversion/clone.
        // Optimization: Implement specialized comparison on RodInput without conversion.
        // For MVP, conversion is acceptable for literals.
        let val = input.to_json();

        if val == self.expected {
            return Ok(RodValue::Json(val));
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
