use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodMap {
    key_type: Box<dyn RodValidator>,
    value_type: Box<dyn RodValidator>,
}

impl RodMap {
    pub fn new(key: Box<dyn RodValidator>, value: Box<dyn RodValidator>) -> Self {
        Self {
            key_type: key,
            value_type: value,
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
            // Path segment for array index
            // We use with_index which uses zero-alloc enum path segment
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

                    // Validate Key (Index 0)
                    let key_res = sub_ctx.with_path("key", |k_ctx| {
                        entry_input.with_index(0, &mut |k_in| {
                            self.key_type.validate_with_context(k_ctx, k_in)
                        })
                    });

                    if key_res.is_none() || key_res.as_ref().unwrap().is_err() {
                        return Err(());
                    }

                    // Validate Value (Index 1)
                    let val_res = sub_ctx.with_path("value", |v_ctx| {
                        entry_input.with_index(1, &mut |v_in| {
                            self.value_type.validate_with_context(v_ctx, v_in)
                        })
                    });

                    if val_res.is_none() || val_res.as_ref().unwrap().is_err() {
                        return Err(());
                    }

                    // Extract values
                    // Safe unwrap because we checked errors
                    let k = key_res.unwrap().unwrap();
                    let v = val_res.unwrap().unwrap();

                    // FIX: Return RodValue to satisfy with_index signature
                    Ok(RodValue::Array(vec![k, v]))
                })
            });

            // Flatten Option<Option<Result>>
            if let Some(Ok(RodValue::Array(mut items))) = entry_result {
                // Unpack the vec to push to valid_entries
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
        let partial_key = self.key_type.deep_partial_boxed();
        let partial_value = self.value_type.deep_partial_boxed();
        Box::new(RodMap::new(partial_key, partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn map(key: impl RodValidator + 'static, value: impl RodValidator + 'static) -> RodMap {
    RodMap::new(Box::new(key), Box::new(value))
}
