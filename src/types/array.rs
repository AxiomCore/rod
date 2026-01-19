use crate::core::input::{DataType, RodInput};
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
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        if input.get_type() == DataType::Array {
            let mut issues = Vec::new();
            let len = input.count().unwrap_or(0);

            // 1. Validate Length
            if let Some(min) = self.min {
                if len < min {
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
                if len > max {
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
            let mut valid_items = Vec::with_capacity(len);
            for i in 0..len {
                if let Some(item_input) = input.get_index(i) {
                    match self.schema.validate(item_input.as_ref()) {
                        Ok(val) => valid_items.push(val),
                        Err(mut e) => {
                            e.prepend_path(&i.to_string());
                            issues.extend(e.issues);
                        }
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
        use crate::types::optional::OptionalExtension;
        let partial_item = self.schema.deep_partial_boxed();
        Box::new(RodArray::new(partial_item).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn array(schema: impl RodValidator + 'static) -> RodArray {
    RodArray::new(Box::new(schema))
}
