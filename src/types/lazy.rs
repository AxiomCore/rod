use crate::core::validator::RodValidator;
use crate::error::RodResult;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

pub struct RodLazy {
    // A function that returns the validator.
    // We cache it internally to avoid rebuilding on every validation?
    // For MVP, we rebuild or use Arc<Box<dyn RodValidator>> if needed.
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
    fn validate(&self, input: &Value) -> RodResult<Value> {
        let validator = (self.builder)();
        validator.validate(input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        // deep_partial of lazy is lazy of deep_partial
        // We create a new lazy validator that calls builder().deep_partial_boxed().
        // BUT lazy() wraps the result in Box<dyn RodValidator>.
        // And deep_partial_boxed() returns Box<dyn RodValidator>.
        // So this works perfectly.
        // We need to clone the builder Arc to move into the closure.
        let builder = self.builder.clone();
        let lazy_partial = RodLazy::new(move || builder().deep_partial_boxed());

        // And wrap in optional? Zod lazy.deepPartial() returns lazy wrapping partial.
        // It doesn't inherently make it optional itself, but usually partial implies optionality.
        // If we follow pattern, we wrap in optional at the end.
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
