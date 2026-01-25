use crate::core::input::{RodInput, RodThreadBounds};
use crate::core::value::RodValue;
use crate::error::{RodResult, ValidationContext};

// FIX: Removed Send + Sync bounds
pub trait RodValidator: std::fmt::Debug + RodThreadBounds {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()>;

    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        let mut ctx = ValidationContext::new();
        let result = self.validate_with_context(&mut ctx, input);

        if ctx.has_issues() {
            return Err(crate::error::RodError { issues: ctx.issues });
        }

        result.map_err(|_| crate::error::RodError { issues: ctx.issues })
    }

    fn is_optional(&self) -> bool {
        false
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator>;
    fn clone_box(&self) -> Box<dyn RodValidator>;
}

impl Clone for Box<dyn RodValidator> {
    fn clone(&self) -> Box<dyn RodValidator> {
        self.clone_box()
    }
}

impl RodValidator for Box<dyn RodValidator> {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        (**self).validate_with_context(ctx, input)
    }

    fn is_optional(&self) -> bool {
        (**self).is_optional()
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        (**self).deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        (**self).clone_box()
    }
}
