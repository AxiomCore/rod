use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodTuple {
    items: Vec<Box<dyn RodValidator>>,
}

impl RodTuple {
    pub fn new(items: Vec<Box<dyn RodValidator>>) -> Self {
        Self { items }
    }
}

impl RodValidator for RodTuple {
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        if input.get_type() != DataType::Array {
            return Err(RodError::new("invalid_type", "Expected tuple (array)"));
        }

        let len = input.count().unwrap_or(0);
        if len != self.items.len() {
            return Err(RodError::new(
                "invalid_tuple_size",
                &format!("Tuple must contain exactly {} elements", self.items.len()),
            ));
        }

        let mut valid_items = Vec::new();
        let mut issues = Vec::new();

        for (i, validator) in self.items.iter().enumerate() {
            if let Some(item_input) = input.get_index(i) {
                match validator.validate(item_input.as_ref()) {
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

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_items = self.items.iter().map(|i| i.deep_partial_boxed()).collect();
        Box::new(RodTuple::new(partial_items).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn tuple(items: Vec<Box<dyn RodValidator>>) -> RodTuple {
    RodTuple::new(items)
}
