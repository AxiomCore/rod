use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use std::fmt;
use std::sync::Arc;

pub struct RodLazy {
    builder: Arc<dyn Fn() -> Box<dyn RodValidator> + Send + Sync>,
}

impl Clone for RodLazy {
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
        }
    }
}

impl fmt::Debug for RodLazy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RodLazy").finish()
    }
}

impl RodLazy {
    pub fn new<F>(builder: F) -> Self
    where
        F: Fn() -> Box<dyn RodValidator> + Send + Sync + 'static,
    {
        Self {
            builder: Arc::new(builder),
        }
    }
}

impl RodValidator for RodLazy {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        let validator = (self.builder)();
        validator.validate_with_context(ctx, input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let builder = self.builder.clone();
        let lazy_partial = RodLazy::new(move || builder().deep_partial_boxed());
        Box::new(lazy_partial.optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn lazy<F>(f: F) -> RodLazy
where
    F: Fn() -> Box<dyn RodValidator> + Send + Sync + 'static,
{
    RodLazy::new(f)
}
