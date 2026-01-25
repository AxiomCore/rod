use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodMap {
    key_type: Box<RodNode>,
    value_type: Box<RodNode>,
}

impl RodMap {
    pub fn new(key: RodNode, value: RodNode) -> Self {
        Self {
            key_type: Box::new(key),
            value_type: Box::new(value),
        }
    }
}

impl RodValidator for RodMap {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Array {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "array (entries)".into(),
                    received: "unknown".into(),
                },
                "Expected array of entries for Map".into(),
            );
            return Err(());
        }

        let len = input.count().unwrap_or(0);
        let mut valid_entries = Vec::with_capacity(len);

        for i in 0..len {
            let entry_result = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |entry_input| {
                    if entry_input.get_type() != DataType::Array {
                        sub_ctx.add_issue(
                            RodIssueCode::InvalidType {
                                expected: "array".into(),
                                received: "unknown".into(),
                            },
                            "Map entries must be arrays".into(),
                        );
                        return Err(());
                    }
                    if entry_input.count() != Some(2) {
                        sub_ctx.add_issue(
                            RodIssueCode::Custom {
                                message: "Map entry must be a [key, value] tuple".into(),
                                params: None,
                            },
                            "Map entry must be a [key, value] tuple".into(),
                        );
                        return Err(());
                    }

                    // STATIC DISPATCH
                    let key_res = sub_ctx.with_path("key", |k_ctx| {
                        entry_input.with_index(0, &mut |k_in| {
                            self.key_type.validate_with_context(k_ctx, k_in)
                        })
                    });

                    if key_res.is_none() || key_res.as_ref().unwrap().is_err() {
                        return Err(());
                    }

                    let val_res = sub_ctx.with_path("value", |v_ctx| {
                        entry_input.with_index(1, &mut |v_in| {
                            self.value_type.validate_with_context(v_ctx, v_in)
                        })
                    });

                    if val_res.is_none() || val_res.as_ref().unwrap().is_err() {
                        return Err(());
                    }

                    let k = key_res.unwrap().unwrap();
                    let v = val_res.unwrap().unwrap();

                    Ok(RodValue::Array(vec![k, v]))
                })
            });

            if let Some(Ok(RodValue::Array(mut items))) = entry_result {
                if items.len() == 2 {
                    let v = items.pop().unwrap();
                    let k = items.pop().unwrap();
                    valid_entries.push(RodValue::Array(vec![k, v]));
                }
            } else if ctx.should_abort() {
                return Err(());
            }
        }

        if ctx.has_issues() {
            return Err(());
        }

        Ok(RodValue::Array(valid_entries))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_key = wrap_custom(self.key_type.deep_partial_boxed());
        let partial_value = wrap_custom(self.value_type.deep_partial_boxed());
        Box::new(RodMap::new(partial_key.into_node(), partial_value.into_node()).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn map<K: IntoRodNode, V: IntoRodNode>(key: K, value: V) -> RodMap {
    RodMap::new(key.into_node(), value.into_node())
}
