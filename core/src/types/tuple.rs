use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodTuple {
    pub items: Vec<RodNode>,
}

impl RodTuple {
    pub fn new(items: Vec<RodNode>) -> Self {
        Self { items }
    }
}

impl RodValidator for RodTuple {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Array {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "tuple".into(),
                    received: "unknown".into(),
                },
                "Expected array".into(),
            );
            return Err(());
        }
        let len = input.count().unwrap_or(0);
        if len != self.items.len() {
            ctx.add_issue(
                RodIssueCode::Custom {
                    message: "Invalid size".into(),
                    params: None,
                },
                "Size mismatch".into(),
            );
            return Err(());
        }

        let mut valid_items = Vec::with_capacity(len);
        for (i, validator) in self.items.iter().enumerate() {
            let item_res = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |item_input| {
                    validator.validate_with_context(sub_ctx, &BoxedInput(item_input))
                })
            });
            if let Some(Ok(val)) = item_res {
                valid_items.push(val);
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
        let partial_items = self
            .items
            .iter()
            .map(|i| wrap_custom(i.deep_partial_boxed()))
            .collect();
        Box::new(RodTuple::new(partial_items).optional())
    }
    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodTuple {
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

pub fn tuple<T: IntoRodNode>(items: Vec<T>) -> RodTuple {
    RodTuple::new(items.into_iter().map(|i| i.into_node()).collect())
}
