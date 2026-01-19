// src/types/boolean.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct RodBoolean;

impl RodValidator for RodBoolean {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if input.is_boolean() {
            return Ok(input.clone());
        }
        Err(RodError::new("invalid_type", "Expected boolean"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn boolean() -> RodBoolean {
    RodBoolean
}
