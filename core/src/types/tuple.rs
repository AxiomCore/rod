use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodTuple {
    items: Vec<Box<dyn RodValidator>>,
}

impl RodTuple {
    pub fn new(items: Vec<Box<dyn RodValidator>>) -> Self {
        Self { items }
    }
}

impl RodValidator for RodTuple {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() != DataType::Array {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "tuple".into(),
                    received: "unknown".into(),
                },
                "Expected tuple (array)".into(),
            );
            return Err(());
        }

        let len = input.count().unwrap_or(0);
        if len != self.items.len() {
            ctx.add_issue(
                RodIssueCode::Custom {
                    message: "Invalid tuple size".into(),
                    params: None,
                },
                format!("Tuple must contain exactly {} elements", self.items.len()),
            );
            return Err(());
        }

        let mut valid_items = Vec::with_capacity(len);

        for (i, validator) in self.items.iter().enumerate() {
            let item_res = ctx.with_index(i, |sub_ctx| {
                input.with_index(i, &mut |item_input| {
                    validator.validate_with_context(sub_ctx, item_input)
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

    fn to_spec(&self) -> RodSpec {
        RodSpec::Tuple {
            items: self.items.iter().map(|i| i.to_spec()).collect(),
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_items = self.items.iter().map(|i| i.deep_partial_boxed()).collect();
        Box::new(RodTuple::new(partial_items).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn tuple(items: Vec<Box<dyn RodValidator>>) -> RodTuple {
    RodTuple::new(items)
}
