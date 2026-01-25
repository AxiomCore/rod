use crate::core::input::{BoxedInput, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{RodNode, wrap_custom};
use std::fmt;
use std::sync::Arc;

pub struct RodLazy {
    pub builder: Arc<dyn Fn() -> RodNode + Send + Sync>,
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
        F: Fn() -> RodNode + Send + Sync + 'static,
    {
        Self {
            builder: Arc::new(builder),
        }
    }
}

impl RodValidator for RodLazy {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        let node = (self.builder)();
        // STATIC DISPATCH through Node enum
        node.validate_with_context(ctx, input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        let builder = self.builder.clone();
        let lazy_partial = RodLazy::new(move || wrap_custom(builder().deep_partial_boxed()));
        Box::new(lazy_partial)
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodLazy {
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

pub fn lazy<F>(f: F) -> RodLazy
where
    F: Fn() -> RodNode + Send + Sync + 'static,
{
    RodLazy::new(f)
}
