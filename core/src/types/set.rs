use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RodSet {
    pub value_type: Box<RodNode>,
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl RodSet {
    pub fn new(value_type: RodNode) -> Self {
        Self {
            value_type: Box::new(value_type),
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
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
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

        for i in 0..len {
            let item_res = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |item_input| {
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
                    self.value_type
                        .validate_with_context(sub_ctx, &BoxedInput(item_input))
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_value = wrap_custom(self.value_type.deep_partial_boxed());
        Box::new(RodSet::new(partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodSet {
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

pub fn set<T: IntoRodNode>(schema: T) -> RodSet {
    RodSet::new(schema.into_node())
}
