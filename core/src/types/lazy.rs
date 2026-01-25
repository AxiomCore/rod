use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{RodNode, wrap_custom};
use std::fmt;
use std::sync::Arc;

pub struct RodLazy {
    // UPDATED: Builder returns a concrete Node
    builder: Arc<dyn Fn() -> RodNode + Send + Sync>,
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
        // F returns RodNode
        F: Fn() -> RodNode + Send + Sync + 'static,
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
        let node = (self.builder)();
        // STATIC DISPATCH
        node.validate_with_context(ctx, input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let builder = self.builder.clone();
        // Wrap the boxed partial into a Custom node to return a Node-compatible Lazy
        let lazy_partial = RodLazy::new(move || wrap_custom(builder().deep_partial_boxed()));
        Box::new(lazy_partial.optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn lazy<F>(f: F) -> RodLazy
where
    F: Fn() -> RodNode + Send + Sync + 'static,
{
    RodLazy::new(f)
}
