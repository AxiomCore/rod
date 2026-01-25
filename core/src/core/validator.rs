use crate::core::input::{RodInput, RodThreadBounds};
use crate::core::value::RodValue;
use crate::error::{RodResult, ValidationContext};

pub trait RodValidator: std::fmt::Debug + RodThreadBounds {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()>;

    fn validate<'a, I: RodInput<'a>>(&self, input: &I) -> RodResult<RodValue<'a>> {
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
    fn deep_partial_boxed(&self) -> Box<dyn DynValidator>;
    fn clone_box(&self) -> Box<dyn DynValidator>;
}

pub trait DynValidator: std::fmt::Debug + RodThreadBounds {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()>;

    fn is_optional_dyn(&self) -> bool {
        false
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator>;
    fn clone_dyn(&self) -> Box<dyn DynValidator>;
}

// Optimization: Removed blanket impl impl<T: RodValidator> DynValidator for T
// to prevent recursive trait loops with Box<dyn DynValidator>.

impl Clone for Box<dyn DynValidator> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl RodValidator for Box<dyn DynValidator> {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        // Direct call to trait object, bypassing the blanket loop
        self.validate_dyn(ctx, input)
    }

    fn is_optional(&self) -> bool {
        self.is_optional_dyn()
    }
    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        self.deep_partial_dyn()
    }
    fn clone_box(&self) -> Box<dyn DynValidator> {
        self.clone_dyn()
    }
}
