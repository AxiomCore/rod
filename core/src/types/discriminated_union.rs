use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RodDiscriminatedUnion {
    pub discriminator: String,
    pub options: HashMap<String, RodNode>,
}

impl RodDiscriminatedUnion {
    pub fn new(discriminator: String, options: HashMap<String, RodNode>) -> Self {
        Self {
            discriminator,
            options,
        }
    }
}

impl RodValidator for RodDiscriminatedUnion {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Object {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "object".into(),
                    received: "unknown".into(),
                },
                "Expected object".into(),
            );
            return Err(());
        }

        let disc_val_opt = input.with_key(&self.discriminator, &mut |disc_input| {
            let wrapper = BoxedInput(disc_input);
            match wrapper.as_str() {
                Some(s) => Ok(RodValue::String(s)),
                None => Err(()),
            }
        });

        let disc_cow = match disc_val_opt {
            Some(Ok(RodValue::String(s))) => s,
            _ => {
                ctx.add_issue(
                    RodIssueCode::InvalidUnionDiscriminator { expected: vec![] },
                    "Discriminator missing or invalid".into(),
                );
                return Err(());
            }
        };

        if let Some(validator) = self.options.get(disc_cow.as_ref()) {
            return validator.validate_with_context(ctx, input);
        }

        ctx.add_issue(
            RodIssueCode::InvalidUnionDiscriminator {
                expected: self.options.keys().cloned().collect(),
            },
            format!("Invalid discriminator: {}", disc_cow),
        );
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodDiscriminatedUnion {
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

pub fn discriminated_union<T: IntoRodNode>(
    discriminator: &str,
    options: Vec<(&str, T)>,
) -> RodDiscriminatedUnion {
    let mut map = HashMap::new();
    for (value, validator) in options {
        map.insert(value.to_string(), validator.into_node());
    }
    RodDiscriminatedUnion::new(discriminator.to_string(), map)
}

pub fn discriminated_union_map(
    discriminator: String,
    options: HashMap<String, RodNode>,
) -> RodDiscriminatedUnion {
    RodDiscriminatedUnion::new(discriminator, options)
}
