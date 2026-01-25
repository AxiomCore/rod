use crate::core::input::{BoxedInput, DataType, RodInput}; // Added BoxedInput
use crate::core::validator::{DynValidator, RodValidator}; // Added DynValidator
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode};

#[derive(Debug, Clone)]
pub struct RodArray {
    pub schema: Box<RodNode>,
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl RodArray {
    pub fn new<T: IntoRodNode>(schema: T) -> Self {
        Self {
            schema: Box::new(schema.into_node()),
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
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Array {
            let len = input.count().unwrap_or(0);

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

            let mut valid_items = Vec::with_capacity(len);
            for i in 0..len {
                let item_res = ctx.with_index(i, |sub_ctx| {
                    input.with_index(i, &mut |item_input| {
                        // Bridge the dynamic item_input to Sized for the Node validation
                        self.schema
                            .validate_with_context(sub_ctx, &BoxedInput(item_input))
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_item = self.schema.deep_partial_boxed();
        let node = crate::types::node::wrap_custom(partial_item);
        Box::new(RodArray::new(node).optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

// NEW: Manual implementation of DynValidator to satisfy RodNode::Custom and break recursion
impl DynValidator for RodArray {
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

pub fn array<T: IntoRodNode>(schema: T) -> RodArray {
    RodArray::new(schema)
}
