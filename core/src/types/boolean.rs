use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone, Default)]
pub struct RodBoolean;

impl RodValidator for RodBoolean {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Boolean {
            let val = input.as_bool().unwrap_or(false);
            return Ok(RodValue::Boolean(val));
        }
        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "boolean".into(),
                received: "unknown".into(),
            },
            "Expected boolean".into(),
        );
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodBoolean {
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

pub fn boolean() -> RodBoolean {
    RodBoolean
}
