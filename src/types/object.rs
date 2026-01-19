use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use crate::types::enum_type::{RodEnum, enum_type};
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnknownKeys {
    Strip,
    Strict,
    Passthrough,
}

#[derive(Clone)]
pub struct RodObject {
    shape: HashMap<String, Box<dyn RodValidator>>,
    unknown_keys: UnknownKeys,
}

// ... impl Debug, new, strict, strip, passthrough, keyof, deep_partial ...
// (These methods don't change, copy from previous implementation)
impl std::fmt::Debug for RodObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RodObject")
            .field("shape_keys", &self.shape.keys())
            .field("unknown_keys", &self.unknown_keys)
            .finish()
    }
}
impl RodObject {
    pub fn new(shape: HashMap<String, Box<dyn RodValidator>>) -> Self {
        Self {
            shape,
            unknown_keys: UnknownKeys::Strip,
        }
    }
    pub fn strict(mut self) -> Self {
        self.unknown_keys = UnknownKeys::Strict;
        self
    }
    pub fn strip(mut self) -> Self {
        self.unknown_keys = UnknownKeys::Strip;
        self
    }
    pub fn passthrough(mut self) -> Self {
        self.unknown_keys = UnknownKeys::Passthrough;
        self
    }

    pub fn keyof(&self) -> RodEnum {
        let keys: Vec<&str> = self.shape.keys().map(|k| k.as_str()).collect();
        enum_type(keys)
    }

    pub fn deep_partial(self) -> RodObject {
        let mut new_shape = HashMap::new();
        for (key, validator) in self.shape {
            new_shape.insert(key, validator.deep_partial_boxed());
        }
        RodObject {
            shape: new_shape,
            unknown_keys: self.unknown_keys,
        }
    }
}

impl RodValidator for RodObject {
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        if input.get_type() == DataType::Object {
            let mut output = Map::new();
            let mut issues = Vec::new();

            // 1. Validate Shape
            for (key, validator) in &self.shape {
                // Try to get key from input
                match input.get_key(key) {
                    Some(val_input) => {
                        // Check for explicit undefined/null if needed?
                        // RodInput usually abstracts this. If get_key returns Some, value exists.
                        match validator.validate(val_input.as_ref()) {
                            Ok(v) => {
                                output.insert(key.clone(), v);
                            }
                            Err(mut e) => {
                                e.prepend_path(key);
                                issues.extend(e.issues);
                            }
                        }
                    }
                    None => {
                        // Missing key
                        if !validator.is_optional() {
                            issues.push(crate::error::RodIssue {
                                details: crate::error::RodIssueCode::InvalidType {
                                    expected: "any".to_string(),
                                    received: "undefined".to_string(),
                                },
                                message: "Required".to_string(),
                                path: vec![key.clone()],
                            });
                        }
                    }
                }
            }

            // 2. Handle Unknown Keys
            // We need to iterate over input keys
            if let Some(keys_iter) = input.keys() {
                for key in keys_iter {
                    if !self.shape.contains_key(&key) {
                        match self.unknown_keys {
                            UnknownKeys::Strict => {
                                issues.push(crate::error::RodIssue {
                                    details: crate::error::RodIssueCode::UnrecognizedKeys {
                                        keys: vec![key.clone()],
                                    },
                                    message: format!("Unrecognized key: '{}'", key),
                                    path: vec![key.clone()],
                                });
                            }
                            UnknownKeys::Passthrough => {
                                // We need to get the value to insert it.
                                if let Some(val_input) = input.get_key(&key) {
                                    // Hack: convert input back to JSON value to insert into output
                                    // This breaks zero-copy for passthrough data, but that's unavoidable for output.
                                    output.insert(key, val_input.to_json());
                                }
                            }
                            UnknownKeys::Strip => {}
                        }
                    }
                }
            }

            if !issues.is_empty() {
                return Err(RodError { issues });
            }

            return Ok(Value::Object(output));
        }

        Err(RodError::new("invalid_type", "Expected object"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().deep_partial().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn object(shape: HashMap<String, Box<dyn RodValidator>>) -> RodObject {
    RodObject::new(shape)
}
