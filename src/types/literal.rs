// src/types/literal.rs
use crate::core::validator::RodValidator;
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
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if input == &self.expected {
            return Ok(input.clone());
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
