use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

// --- ANY ---
#[derive(Debug, Clone, Default)]
pub struct RodAny;

impl RodValidator for RodAny {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        Ok(input.clone())
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn any() -> RodAny {
    RodAny
}

// --- NEVER ---
#[derive(Debug, Clone, Default)]
pub struct RodNever;

impl RodValidator for RodNever {
    fn validate(&self, _input: &Value) -> RodResult<Value> {
        Err(RodError::new(
            "invalid_type",
            "Expected never, received something",
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

pub fn never() -> RodNever {
    RodNever
}
