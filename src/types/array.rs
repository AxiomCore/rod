// src/types/array.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodArray {
    schema: Box<dyn RodValidator>,
    min: Option<usize>,
    max: Option<usize>,
}

impl RodArray {
    pub fn new(schema: Box<dyn RodValidator>) -> Self {
        Self {
            schema,
            min: None,
            max: None,
        }
    }

    pub fn min(mut self, val: usize) -> Self {
        self.min = Some(val);
        self
    }

    pub fn max(mut self, val: usize) -> Self {
        self.max = Some(val);
        self
    }

    pub fn nonempty(mut self) -> Self {
        self.min = Some(1);
        self
    }
}

impl RodValidator for RodArray {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if let Value::Array(arr) = input {
            let mut issues = Vec::new();

            // 1. Validate Length
            if let Some(min) = self.min {
                if arr.len() < min {
                    issues.push(crate::error::RodIssue {
                        details: crate::error::RodIssueCode::TooSmall {
                            minimum: min as f64,
                            inclusive: true,
                            type_: "array".to_string(),
                        },
                        message: format!("Array must contain at least {} element(s)", min),
                        path: vec![],
                    });
                }
            }
            if let Some(max) = self.max {
                if arr.len() > max {
                    issues.push(crate::error::RodIssue {
                        details: crate::error::RodIssueCode::TooBig {
                            maximum: max as f64,
                            inclusive: true,
                            type_: "array".to_string(),
                        },
                        message: format!("Array must contain at most {} element(s)", max),
                        path: vec![],
                    });
                }
            }

            // 2. Validate Items
            let mut valid_items = Vec::new();
            for (index, item) in arr.iter().enumerate() {
                match self.schema.validate(item) {
                    Ok(val) => valid_items.push(val),
                    Err(mut e) => {
                        // Bubbling up error: Add index to path (e.g. "users.0.name")
                        e.prepend_path(&index.to_string());
                        issues.extend(e.issues);
                    }
                }
            }

            if !issues.is_empty() {
                return Err(RodError { issues });
            }

            return Ok(Value::Array(valid_items));
        }

        Err(RodError::new("invalid_type", "Expected array"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        // Just return optional version of self
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn array(schema: impl RodValidator + 'static) -> RodArray {
    RodArray::new(Box::new(schema))
}
