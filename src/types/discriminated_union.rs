use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};
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
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // 1. Check if input is an object
        if input.get_type() != DataType::Object {
            return Err(RodError::new("invalid_type", "Expected object"));
        }

        // 2. Extract discriminator value
        // We use the trait method get_key
        let disc_input = match input.get_key(&self.discriminator) {
            Some(v) => v,
            None => {
                // Try checking raw if needed? RodInput usually handles this.
                return Err(RodError::new(
                    "invalid_discriminator",
                    "Discriminator key missing",
                ));
            }
        };

        // Ensure it is a string
        // Note: as_str might return Option<&str> or Option<Cow<str>>
        let disc_value = match disc_input.as_str() {
            Some(s) => s,
            None => {
                return Err(RodError::new(
                    "invalid_discriminator",
                    "Discriminator value must be a string",
                ));
            }
        };

        // 3. Select matching validator
        if let Some(validator) = self.options.get(disc_value.as_ref()) {
            return validator.validate(input);
        }

        Err(RodError::new(
            "invalid_union_discriminator",
            &format!("Invalid discriminator value: {}", disc_value),
        ))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        // Cannot easily deep partial a discriminated union while keeping discriminator required.
        // Returning optional of self.
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
