use crate::RodSpec;
use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

// ANY
#[derive(Debug, Clone, Default)]
pub struct RodAny;
impl RodValidator for RodAny {
    fn validate_with_context<'a>(
        &self,
        _ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // Zero-Copy Pass-through using Lazy cursor
        Ok(RodValue::Lazy(input.clone_box()))
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Any
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }
    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}
pub fn any() -> RodAny {
    RodAny
}

// NEVER
#[derive(Debug, Clone, Default)]
pub struct RodNever;
impl RodValidator for RodNever {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        _input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "never".into(),
                received: "any".into(),
            },
            "Expected never".into(),
        );
        Err(())
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Never
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }
    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}
pub fn never() -> RodNever {
    RodNever
}
