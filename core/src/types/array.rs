use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode};

#[derive(Debug, Clone)]
pub struct RodArray {
    pub schema: Box<RodNode>,
    min: Option<usize>,
    max: Option<usize>,
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
                        // HYBRID DISPATCH: Calling validate on RodNode enum variant
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
        // Since deep_partial_boxed returns Box<dyn Validator>, we lose static info here during recursion logic.
        // The resulting array will contain a Custom(Box<dyn>) node.
        // This is acceptable for 'partial' schemas which are less performance critical than the main schema.
        let partial_item = self.schema.deep_partial_boxed();

        // We need to wrap the boxed validator into a Node::Custom to create a RodArray
        let node = crate::types::node::wrap_custom(partial_item);

        // Return Array<Custom> wrapped in Optional
        Box::new(RodArray::new(node).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn array<T: IntoRodNode>(schema: T) -> RodArray {
    RodArray::new(schema)
}
