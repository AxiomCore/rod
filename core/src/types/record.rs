use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::io::json;
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodRecord {
    pub key_schema: Box<RodNode>,
    pub value_schema: Box<RodNode>,
}

impl RodRecord {
    pub fn new(key_schema: RodNode, value_schema: RodNode) -> Self {
        Self {
            key_schema: Box::new(key_schema),
            value_schema: Box::new(value_schema),
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

                let key_valid = ctx.with_owned_path(key.to_string(), |sub_ctx| {
                    // STATIC DISPATCH
                    self.key_schema.validate_with_context(sub_ctx, &key_input)
                });

                if key_valid.is_err() {
                    if ctx.should_abort() {
                        return Err(());
                    }
                    continue;
                }

                let val_res = ctx.with_owned_path(key.to_string(), |sub_ctx| {
                    input.with_key(key.as_ref(), &mut |val_input| {
                        // STATIC DISPATCH
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
        let partial_value = wrap_custom(self.value_schema.deep_partial_boxed());
        Box::new(RodRecord::new((*self.key_schema).clone(), partial_value.into_node()).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn record<K: IntoRodNode, V: IntoRodNode>(key: K, value: V) -> RodRecord {
    RodRecord::new(key.into_node(), value.into_node())
}
