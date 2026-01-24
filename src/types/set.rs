use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RodSet {
    value_type: Box<dyn RodValidator>,
    min: Option<usize>,
    max: Option<usize>,
}

impl RodSet {
    pub fn new(value_type: Box<dyn RodValidator>) -> Self {
        Self {
            value_type,
            min: None,
            max: None,
        }
    }
    pub fn min(mut self, v: usize) -> Self {
        self.min = Some(v);
        self
    }
    pub fn max(mut self, v: usize) -> Self {
        self.max = Some(v);
        self
    }
}

impl RodValidator for RodSet {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Array {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "array (set)".into(),
                    received: "unknown".into(),
                },
                "Expected array (set)".into(),
            );
            return Err(());
        }

        let len = input.count().unwrap_or(0);
        let mut valid_items = Vec::with_capacity(len);
        let mut seen = HashSet::new();

        if let Some(min) = self.min {
            if len < min {
                ctx.add_issue(
                    RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "set".into(),
                    },
                    format!("Set too small"),
                );
            }
        }
        if let Some(max) = self.max {
            if len > max {
                ctx.add_issue(
                    RodIssueCode::TooBig {
                        maximum: max as f64,
                        inclusive: true,
                        type_: "set".into(),
                    },
                    format!("Set too big"),
                );
            }
        }

        for i in 0..len {
            let item_res = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |item_input| {
                    // Uniqueness Check via JSON string
                    let s = item_input.to_json().to_string();
                    if !seen.insert(s) {
                        sub_ctx.add_issue(
                            RodIssueCode::Custom {
                                message: "Items must be unique".into(),
                                params: None,
                            },
                            "Items must be unique".into(),
                        );
                    }
                    self.value_type.validate_with_context(sub_ctx, item_input)
                })
            });

            if let Some(Ok(v)) = item_res {
                valid_items.push(v);
            } else if ctx.should_abort() {
                return Err(());
            }
        }

        if ctx.has_issues() {
            return Err(());
        }

        Ok(RodValue::Array(valid_items))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_value = self.value_type.deep_partial_boxed();
        Box::new(RodSet::new(partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn set(schema: impl RodValidator + 'static) -> RodSet {
    RodSet::new(Box::new(schema))
}
