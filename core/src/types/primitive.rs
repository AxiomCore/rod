use crate::core::input::{BoxedInput, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

// ANY
#[derive(Debug, Clone, Default)]
pub struct RodAny;
impl RodValidator for RodAny {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        _ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        Ok(RodValue::Lazy(input.clone_box()))
    }
    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }
    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}
impl DynValidator for RodAny {
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
pub fn any() -> RodAny {
    RodAny
}

// NEVER
#[derive(Debug, Clone, Default)]
pub struct RodNever;
impl RodValidator for RodNever {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        _input: &I,
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
    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }
    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}
impl DynValidator for RodNever {
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
pub fn never() -> RodNever {
    RodNever
}
