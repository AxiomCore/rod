use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};
use crate::io::json; // used to create JsonInput for keys

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
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        if input.get_type() != DataType::Object {
            return Err(RodError::new("invalid_type", "Expected object (record)"));
        }

        let mut output = Vec::new();
        let mut issues = Vec::new();

        // Iterate over keys using the trait
        if let Some(keys) = input.keys() {
            for key in keys {
                // 1. Validate Key
                // Keys are strings. To validate using RodValidator, we need to wrap the string key
                // into a RodInput. The easiest way is using our JsonInput wrapper.
                let key_val = serde_json::Value::String(key.clone());
                let key_input = json::wrap(&key_val);

                if let Err(mut e) = self.key_schema.validate(&key_input) {
                    e.prepend_path(&key);
                    issues.extend(e.issues);
                    continue;
                }

                // 2. Validate Value
                if let Some(val_input) = input.get_key(&key) {
                    match self.value_schema.validate(val_input.as_ref()) {
                        Ok(val) => {
                            output.push((std::borrow::Cow::Owned(key.clone()), val.into_owned()));
                        }
                        Err(mut e) => {
                            e.prepend_path(&key);
                            issues.extend(e.issues);
                        }
                    }
                }
            }
        }

        if !issues.is_empty() {
            return Err(RodError { issues });
        }

        return Ok(RodValue::Object(output));
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
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
