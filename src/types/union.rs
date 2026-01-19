// src/types/union.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodUnion {
    options: Vec<Box<dyn RodValidator>>,
}

impl RodUnion {
    pub fn new(options: Vec<Box<dyn RodValidator>>) -> Self {
        Self { options }
    }
}

impl RodValidator for RodUnion {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        // Try each option in order
        for validator in &self.options {
            if let Ok(val) = validator.validate(input) {
                return Ok(val);
            }
        }

        // If we get here, no schema matched
        // Zod actually returns specific union errors, but for MVP we return a general error
        // describing that none matched.
        Err(RodError::new("invalid_union", "Invalid input"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_options = self
            .options
            .iter()
            .map(|o| o.deep_partial_boxed())
            .collect();
        Box::new(RodUnion::new(partial_options).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn union(options: Vec<Box<dyn RodValidator>>) -> RodUnion {
    RodUnion::new(options)
}
