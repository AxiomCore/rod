use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodArray {
    schema: Box<dyn RodValidator>,
    min: Option<usize>,
    max: Option<usize>,
}

impl RodArray {
    pub fn new(schema: Box<dyn RodValidator>) -> Self {
        Self {
            schema,
            min: None,
            max: None,
        }
    }
    pub fn min(mut self, val: usize) -> Self {
        self.min = Some(val);
        self
    }
    pub fn max(mut self, val: usize) -> Self {
        self.max = Some(val);
        self
    }
    pub fn nonempty(mut self) -> Self {
        self.min = Some(1);
        self
    }
}

impl RodValidator for RodArray {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Array {
            let len = input.count().unwrap_or(0);

            // 1. Validate Length
            if let Some(min) = self.min {
                if len < min {
                    ctx.add_issue(
                        RodIssueCode::TooSmall {
                            minimum: min as f64,
                            inclusive: true,
                            type_: "array".into(),
                        },
                        format!("Array must contain at least {} element(s)", min),
                    );
                }
            }
            if let Some(max) = self.max {
                if len > max {
                    ctx.add_issue(
                        RodIssueCode::TooBig {
                            maximum: max as f64,
                            inclusive: true,
                            type_: "array".into(),
                        },
                        format!("Array must contain at most {} element(s)", max),
                    );
                }
            }

            // 2. Validate Items using with_index visitor (Zero Copy)
            let mut valid_items = Vec::with_capacity(len);
            for i in 0..len {
                let item_res = ctx.with_index(i, |sub_ctx| {
                    input.with_index(i, &mut |item_input| {
                        self.schema.validate_with_context(sub_ctx, item_input)
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

            return Ok(RodValue::Array(valid_items));
        }

        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "array".into(),
                received: "unknown".into(),
            },
            "Expected array".into(),
        );
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_item = self.schema.deep_partial_boxed();
        Box::new(RodArray::new(partial_item).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn array(schema: impl RodValidator + 'static) -> RodArray {
    RodArray::new(Box::new(schema))
}
