use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;
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
    fn validate(&self, input: &Value) -> RodResult<Value> {
        // 1. Check if input is an object
        let obj = match input {
            Value::Object(o) => o,
            _ => return Err(RodError::new("invalid_type", "Expected object")),
        };

        // 2. Extract discriminator value
        let disc_value = match obj.get(&self.discriminator) {
            Some(Value::String(s)) => s,
            Some(_) => {
                return Err(RodError::new(
                    "invalid_discriminator",
                    "Discriminator value must be a string",
                ));
            }
            None => {
                return Err(RodError::new(
                    "invalid_discriminator",
                    "Discriminator key missing",
                ));
            }
        };

        // 3. Select matching validator
        if let Some(validator) = self.options.get(disc_value) {
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

/// Creates a discriminated union validator.
///
/// Unlike Zod (which introspects schemas), Rust requires explicit mapping of
/// literal values to schemas.
///
/// # Example
/// ```rust
/// use rod::{discriminated_union, rod_obj, literal, string};
///
/// let schema = discriminated_union("type", vec![
///     ("user", Box::new(rod_obj! { type: literal("user"), name: string() })),
///     ("admin", Box::new(rod_obj! { type: literal("admin"), role: string() })),
/// ]);
/// ```
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

// Low-level constructor if you already have a HashMap
pub fn discriminated_union_map(
    discriminator: String,
    options: HashMap<String, Box<dyn RodValidator>>,
) -> RodDiscriminatedUnion {
    RodDiscriminatedUnion::new(discriminator, options)
}
