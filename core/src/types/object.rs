use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::enum_type::{RodEnum, enum_type};
use crate::types::node::{IntoRodNode, RodNode};
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
    pub shape: HashMap<String, RodNode>,
    pub unknown_keys: UnknownKeys,
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
    pub fn new(shape: HashMap<String, RodNode>) -> Self {
        Self {
            shape,
            unknown_keys: UnknownKeys::Strip,
        }
    }

    pub fn from_validators<T: IntoRodNode>(map: HashMap<String, T>) -> Self {
        let mut shape = HashMap::new();
        for (k, v) in map {
            shape.insert(k, v.into_node());
        }
        Self::new(shape)
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
            new_shape.insert(
                key,
                crate::types::node::wrap_custom(validator.deep_partial_boxed()),
            );
        }
        RodObject {
            shape: new_shape,
            unknown_keys: self.unknown_keys,
        }
    }
}

impl RodValidator for RodObject {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Object {
            let mut output = Vec::new();

            for (key, validator) in &self.shape {
                let result_op = ctx.with_owned_path(key.clone(), |sub_ctx| {
                    input.with_key(key, &mut |child_input| {
                        validator.validate_with_context(sub_ctx, &BoxedInput(child_input))
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
                                    if let Some(Ok(v)) =
                                        input.with_key(key.as_ref(), &mut |field_input| {
                                            Ok(RodValue::Lazy(field_input.clone_box()))
                                        })
                                    {
                                        output.push((key, v));
                                    }
                                }
                                UnknownKeys::Strip => {}
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().deep_partial().optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodObject {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

pub fn object(shape: HashMap<String, Box<dyn DynValidator>>) -> RodObject {
    let mut node_shape = HashMap::new();
    for (k, v) in shape {
        node_shape.insert(k, RodNode::Custom(v));
    }
    RodObject::new(node_shape)
}
