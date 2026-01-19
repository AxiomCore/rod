// src/types/number.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodIssue, RodResult};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct RodNumber {
    min: Option<f64>,
    max: Option<f64>,
    is_int: bool,
}

impl RodNumber {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min(mut self, val: f64) -> Self {
        self.min = Some(val);
        self
    }

    pub fn max(mut self, val: f64) -> Self {
        self.max = Some(val);
        self
    }

    pub fn int(mut self) -> Self {
        self.is_int = true;
        self
    }
}

impl RodValidator for RodNumber {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if let Value::Number(n) = input {
            let val = n
                .as_f64()
                .ok_or_else(|| RodError::new("invalid_number", "Invalid number"))?;
            let mut issues = Vec::new();

            if self.is_int && !n.is_i64() && (val.fract() != 0.0) {
                issues.push(RodIssue {
                    details: crate::error::RodIssueCode::InvalidType {
                        expected: "integer".to_string(),
                        received: "float".to_string(),
                    },
                    message: "Expected integer, received float".to_string(),
                    path: vec![],
                });
            }

            if let Some(min) = self.min {
                if val < min {
                    issues.push(RodIssue {
                        details: crate::error::RodIssueCode::TooSmall {
                            minimum: min,
                            inclusive: true, // Assuming inclusive
                            type_: "number".to_string(),
                        },
                        message: format!("Number must be greater than or equal to {}", min),
                        path: vec![],
                    });
                }
            }

            if let Some(max) = self.max {
                if val > max {
                    issues.push(RodIssue {
                        details: crate::error::RodIssueCode::TooBig {
                            maximum: max,
                            inclusive: true, // Assuming inclusive
                            type_: "number".to_string(),
                        },
                        message: format!("Number must be less than or equal to {}", max),
                        path: vec![],
                    });
                }
            }

            if !issues.is_empty() {
                return Err(RodError { issues });
            }

            return Ok(input.clone());
        }

        Err(RodError::new("invalid_type", "Expected number"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn number() -> RodNumber {
    RodNumber::new()
}
