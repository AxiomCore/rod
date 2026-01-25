use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodMap {
    pub key_type: Box<RodNode>,
    pub value_type: Box<RodNode>,
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
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Array {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "array (entries)".into(),
                    received: "unknown".into(),
                },
                "Expected array of entries".into(),
            );
            return Err(());
        }

        let len = input.count().unwrap_or(0);
        let mut valid_entries = Vec::with_capacity(len);

        for i in 0..len {
            let entry_result = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |entry_input| {
                    let key_res = sub_ctx.with_path("key", |k_ctx| {
                        entry_input.with_index(0, &mut |k_in| {
                            self.key_type
                                .validate_with_context(k_ctx, &BoxedInput(k_in))
                        })
                    });

                    let val_res = sub_ctx.with_path("value", |v_ctx| {
                        entry_input.with_index(1, &mut |v_in| {
                            self.value_type
                                .validate_with_context(v_ctx, &BoxedInput(v_in))
                        })
                    });

                    match (key_res, val_res) {
                        (Some(Ok(k)), Some(Ok(v))) => Ok(RodValue::Array(vec![k, v])),
                        _ => Err(()),
                    }
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        let partial_key = wrap_custom(self.key_type.deep_partial_boxed());
        let partial_value = wrap_custom(self.value_type.deep_partial_boxed());
        Box::new(RodMap::new(
            partial_key.into_node(),
            partial_value.into_node(),
        ))
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodMap {
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

pub fn map<K: IntoRodNode, V: IntoRodNode>(key: K, value: V) -> RodMap {
    RodMap::new(key.into_node(), value.into_node())
}
