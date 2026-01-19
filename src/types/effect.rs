use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::error::{RodError, RodResult};
use serde_json::Value;
use std::fmt;

// --- Refine ---

#[derive(Clone)]
pub struct RodRefine<F>
where
    F: Fn(&Value) -> Result<(), String> + Send + Sync + Clone,
{
    schema: Box<dyn RodValidator>,
    check: F,
}

impl<F> fmt::Debug for RodRefine<F>
where
    F: Fn(&Value) -> Result<(), String> + Send + Sync + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RodRefine")
            .field("schema", &self.schema)
            .field("check", &"<function>")
            .finish()
    }
}

impl<F> RodRefine<F>
where
    F: Fn(&Value) -> Result<(), String> + Send + Sync + Clone,
{
    pub fn new(schema: Box<dyn RodValidator>, check: F) -> Self {
        Self { schema, check }
    }
}

impl<F> RodValidator for RodRefine<F>
where
    F: Fn(&Value) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        // 1. Validate Inner (creates owned Value)
        let val = self.schema.validate(input)?;
        // 2. Check on Value
        if let Err(msg) = (self.check)(&val) {
            return Err(RodError::new("custom_error", &msg));
        }
        Ok(val)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        // Zod says "Zod effects are NOT preserving refinements on deepPartial".
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

// --- Transform ---

#[derive(Clone)]
pub struct RodTransform<F>
where
    F: Fn(Value) -> Value + Send + Sync + Clone,
{
    schema: Box<dyn RodValidator>,
    transformer: F,
}

impl<F> fmt::Debug for RodTransform<F>
where
    F: Fn(Value) -> Value + Send + Sync + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RodTransform")
            .field("schema", &self.schema)
            .field("transformer", &"<function>")
            .finish()
    }
}

impl<F> RodTransform<F>
where
    F: Fn(Value) -> Value + Send + Sync + Clone,
{
    pub fn new(schema: Box<dyn RodValidator>, transformer: F) -> Self {
        Self {
            schema,
            transformer,
        }
    }
}

impl<F> RodValidator for RodTransform<F>
where
    F: Fn(Value) -> Value + Send + Sync + Clone + 'static,
{
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        let val = self.schema.validate(input)?;
        Ok((self.transformer)(val))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        // Transforms lost on deep partial
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

// --- Helpers ---

pub fn refine<V, F>(validator: V, check: F) -> RodRefine<F>
where
    V: RodValidator + 'static,
    F: Fn(&Value) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    RodRefine::new(Box::new(validator), check)
}

pub fn transform<V, F>(validator: V, transformer: F) -> RodTransform<F>
where
    V: RodValidator + 'static,
    F: Fn(Value) -> Value + Send + Sync + Clone + 'static,
{
    RodTransform::new(Box::new(validator), transformer)
}
