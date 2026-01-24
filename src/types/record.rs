use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::io::json;

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
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Object {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "object (record)".into(),
                    received: "unknown".into(),
                },
                "Expected object".into(),
            );
            return Err(());
        }

        let mut output = Vec::new();

        if let Some(keys) = input.keys() {
            for key in keys {
                let key_val = serde_json::Value::String(key.to_string());
                let key_input = json::wrap(&key_val);

                // FIX: Use with_owned_path
                let key_valid = ctx.with_owned_path(key.to_string(), |sub_ctx| {
                    self.key_schema.validate_with_context(sub_ctx, &key_input)
                });

                if key_valid.is_err() {
                    if ctx.should_abort() {
                        return Err(());
                    }
                    continue;
                }

                // 2. Validate Value
                // FIX: Use with_owned_path
                let val_res = ctx.with_owned_path(key.to_string(), |sub_ctx| {
                    input.with_key(key.as_ref(), &mut |val_input| {
                        self.value_schema.validate_with_context(sub_ctx, val_input)
                    })
                });

                if let Some(Ok(val)) = val_res {
                    output.push((key, val));
                } else if ctx.should_abort() {
                    return Err(());
                }
            }
        }

        if ctx.has_issues() {
            return Err(());
        }

        Ok(RodValue::Object(output))
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
