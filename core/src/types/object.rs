use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::enum_type::{RodEnum, enum_type};
use std::borrow::Cow;
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
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Object {
            let mut output = Vec::new();

            // 1. Validate Shape
            for (key, validator) in &self.shape {
                // FIX: Use with_owned_path (clone key) to avoid lifetime issues
                // between self (schema) and ctx (ephemeral).
                let result_op = ctx.with_owned_path(key.clone(), |sub_ctx| {
                    input.with_key(key, &mut |child_input| {
                        validator.validate_with_context(sub_ctx, child_input)
                    })
                });

                match result_op {
                    Some(Ok(v)) => {
                        output.push((Cow::Owned(key.clone()), v));
                    }
                    Some(Err(_)) => {
                        if ctx.should_abort() {
                            return Err(());
                        }
                    }
                    None => {
                        if !validator.is_optional() {
                            // FIX: Use with_owned_path for error reporting too
                            ctx.with_owned_path(key.clone(), |sub_ctx| {
                                sub_ctx.add_issue(
                                    RodIssueCode::InvalidType {
                                        expected: "any".into(),
                                        received: "undefined".into(),
                                    },
                                    "Required".into(),
                                );
                            });
                        }
                    }
                }
            }

            // 2. Handle Unknown Keys
            // OPTIMIZATION: Only pay the cost of iterating keys if we actually care about them.
            // If UnknownKeys::Strip (default), we do nothing with extra keys, so we skip this entirely.
            if self.unknown_keys != UnknownKeys::Strip {
                if let Some(keys_iter) = input.keys() {
                    for key in keys_iter {
                        if !self.shape.contains_key(key.as_ref()) {
                            match self.unknown_keys {
                                UnknownKeys::Strict => {
                                    ctx.with_owned_path(key.to_string(), |sub_ctx| {
                                        sub_ctx.add_issue(
                                            RodIssueCode::UnrecognizedKeys {
                                                keys: vec![key.to_string()],
                                            },
                                            format!("Unrecognized key: '{}'", key),
                                        );
                                    });
                                }
                                UnknownKeys::Passthrough => {
                                    let val_lazy =
                                        input.with_key(key.as_ref(), &mut |field_input| {
                                            Ok(RodValue::Lazy(field_input.clone_box()))
                                        });

                                    if let Some(Ok(v)) = val_lazy {
                                        output.push((key, v));
                                    }
                                }
                                UnknownKeys::Strip => {} // Should not be reached
                            }
                        }
                    }
                }
            }

            if ctx.has_issues() {
                return Err(());
            }

            return Ok(RodValue::Object(output));
        }

        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "object".into(),
                received: "unknown".into(),
            },
            "Expected object".into(),
        );
        Err(())
    }

    fn to_spec(&self) -> RodSpec {
        let mut properties = std::collections::HashMap::new();
        for (name, validator) in &self.shape {
            properties.insert(name.clone(), validator.to_spec());
        }
        RodSpec::Object {
            properties,
            strict: Some(self.unknown_keys == UnknownKeys::Strict),
        }
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
