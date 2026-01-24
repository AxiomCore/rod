use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RodDiscriminatedUnion {
    discriminator: String,
    options: HashMap<String, Box<dyn RodValidator>>,
}

impl RodDiscriminatedUnion {
    pub fn new(discriminator: String, options: HashMap<String, Box<dyn RodValidator>>) -> Self {
        Self {
            discriminator,
            options,
        }
    }
}

impl RodValidator for RodDiscriminatedUnion {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
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

        // Use with_key to get discriminator
        let disc_val_opt = input.with_key(&self.discriminator, &mut |disc_input| {
            // We need to return the value to decide which schema to use.
            // But with_key expects Result<RodValue>.
            // We return the extracted string as RodValue::String for processing.
            match disc_input.as_str() {
                Some(s) => Ok(RodValue::String(s)),
                None => Err(()), // Signal failure to extract string
            }
        });

        // Flatten Option<Result<RodValue>>
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
            format!("Invalid discriminator value: {}", disc_cow),
        );
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn discriminated_union(
    discriminator: &str,
    options: Vec<(&str, Box<dyn RodValidator>)>,
) -> RodDiscriminatedUnion {
    let mut map = HashMap::new();
    for (value, validator) in options {
        map.insert(value.to_string(), validator);
    }
    RodDiscriminatedUnion::new(discriminator.to_string(), map)
}

pub fn discriminated_union_map(
    discriminator: String,
    options: HashMap<String, Box<dyn RodValidator>>,
) -> RodDiscriminatedUnion {
    RodDiscriminatedUnion::new(discriminator, options)
}
