// src/types/object.rs
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
            unknown_keys: UnknownKeys::Strip, // Default Zod behavior
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

    /// Creates a new RodObject where all properties (recursive) are optional
    pub fn deep_partial(self) -> RodObject {
        let mut new_shape = HashMap::new();

        for (key, validator) in self.shape {
            // This is the tricky part: Rust trait objects don't allow easy downcasting or introspection
            // without Any or specific methods.
            // A true deep_partial needs to know if the child is an Object to recurse.

            // For this MVP implementation in a static typed language without reflection on trait objects:
            // 1. We wrap the current validator in Optional (shallow partial).
            // 2. To do DEEP partial, we'd need RodValidator to have a `deep_partial()` method
            //    in the trait definition itself that returns Box<dyn RodValidator>.

            // IMPLEMENTATION STRATEGY:
            // We will modify the RodValidator trait to include `fn deep_partial(&self) -> Box<dyn RodValidator>`.
            // Default impl just returns `optional()`. RodObject overrides it to recurse.

            new_shape.insert(key, validator.deep_partial_boxed());
        }

        RodObject {
            shape: new_shape,
            unknown_keys: self.unknown_keys,
        }
    }
}

impl RodValidator for RodObject {
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if let Value::Object(obj) = input {
            let mut output = Map::new();
            let mut issues = Vec::new();

            // 1. Validate Shape
            for (key, validator) in &self.shape {
                match obj.get(key) {
                    Some(val) => match validator.validate(val) {
                        Ok(v) => {
                            output.insert(key.clone(), v);
                        }
                        Err(mut e) => {
                            e.prepend_path(key);
                            issues.extend(e.issues);
                        }
                    },
                    None => {
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
            for (key, val) in obj {
                if !self.shape.contains_key(key) {
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
                            output.insert(key.clone(), val.clone());
                        }
                        UnknownKeys::Strip => {
                            // Do nothing (don't add to output)
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
