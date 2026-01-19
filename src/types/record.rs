// src/types/record.rs
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct RodRecord {
    key_schema: Box<dyn RodValidator>,
    value_schema: Box<dyn RodValidator>,
}

impl RodRecord {
    pub fn new(key_schema: Box<dyn RodValidator>, value_schema: Box<dyn RodValidator>) -> Self {
        Self {
            key_schema,
            value_schema,
        }
    }
}

impl RodValidator for RodRecord {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if let Value::Object(obj) = input {
            let mut output = Map::new();
            let mut issues = Vec::new();

            for (key, value) in obj {
                // Validate Key
                // Keys in JSON are always strings, but we wrap them in Value::String
                // to use the full power of RodValidator (e.g. regex, min length)
                let key_val = Value::String(key.clone());
                if let Err(mut e) = self.key_schema.validate(&key_val) {
                    e.prepend_path(key); // Mark error location
                    issues.extend(e.issues);
                    continue; // Skip value validation if key is bad
                }

                // Validate Value
                match self.value_schema.validate(value) {
                    Ok(val) => {
                        output.insert(key.clone(), val);
                    }
                    Err(mut e) => {
                        e.prepend_path(key);
                        issues.extend(e.issues);
                    }
                }
            }

            if !issues.is_empty() {
                return Err(RodError { issues });
            }

            return Ok(Value::Object(output));
        }

        Err(RodError::new("invalid_type", "Expected object (record)"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        // Zod record keys are not partialed?
        let partial_value = self.value_schema.deep_partial_boxed();
        Box::new(RodRecord::new(self.key_schema.clone_box(), partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn record(key: impl RodValidator + 'static, value: impl RodValidator + 'static) -> RodRecord {
    RodRecord::new(Box::new(key), Box::new(value))
}
