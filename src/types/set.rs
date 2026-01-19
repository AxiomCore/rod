use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RodSet {
    value_type: Box<dyn RodValidator>,
    min: Option<usize>,
    max: Option<usize>,
}

impl RodSet {
    pub fn new(value_type: Box<dyn RodValidator>) -> Self {
        Self {
            value_type,
            min: None,
            max: None,
        }
    }
    pub fn min(mut self, v: usize) -> Self {
        self.min = Some(v);
        self
    }
    pub fn max(mut self, v: usize) -> Self {
        self.max = Some(v);
        self
    }
}

impl RodValidator for RodSet {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        if input.get_type() != DataType::Array {
            return Err(RodError::new("invalid_type", "Expected array (set)"));
        }

        let mut issues = Vec::new();
        let mut valid_items = Vec::new();
        let mut seen = HashSet::new();
        let len = input.count().unwrap_or(0);

        for i in 0..len {
            if let Some(item_input) = input.get_index(i) {
                // Check Uniqueness using json string representation
                let s = item_input.to_json().to_string();
                if !seen.insert(s) {
                    issues.push(crate::error::RodIssue {
                        details: crate::error::RodIssueCode::Custom {
                            message: "Items must be unique".to_string(),
                            params: None,
                        },
                        message: "Items must be unique".to_string(),
                        path: vec![i.to_string()],
                    });
                }

                match self.value_type.validate(item_input.as_ref()) {
                    Ok(v) => valid_items.push(v.into_owned()),
                    Err(mut e) => {
                        e.prepend_path(&i.to_string());
                        issues.extend(e.issues);
                    }
                }
            }
        }

        // Size checks
        if let Some(min) = self.min {
            if len < min {
                issues.push(crate::error::RodIssue {
                    details: crate::error::RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "set".to_string(),
                    },
                    message: format!("Set must contain at least {} items", min),
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
                        type_: "set".to_string(),
                    },
                    message: format!("Set must contain at most {} element(s)", max),
                    path: vec![],
                });
            }
        }

        if !issues.is_empty() {
            return Err(RodError { issues });
        }

        // Return as Array (JSON doesn't have Set type)
        return Ok(RodValue::Array(valid_items));
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_value = self.value_type.deep_partial_boxed();
        Box::new(RodSet::new(partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn set(schema: impl RodValidator + 'static) -> RodSet {
    RodSet::new(Box::new(schema))
}
