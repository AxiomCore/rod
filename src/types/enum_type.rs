// src/types/enum_type.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodEnum {
    values: Vec<String>,
}

impl RodEnum {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
}

impl RodValidator for RodEnum {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if let Value::String(s) = input {
            if self.values.contains(s) {
                return Ok(input.clone());
            }
        }

        Err(RodError::new(
            "invalid_enum_value",
            &format!("Invalid enum value. Expected one of: {:?}", self.values),
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

pub fn enum_type(values: Vec<&str>) -> RodEnum {
    RodEnum::new(values.iter().map(|s| s.to_string()).collect())
}
