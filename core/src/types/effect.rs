use crate::RodSpec;
use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use std::fmt;

// --- Refine ---

#[derive(Clone)]
pub struct RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone,
{
    schema: Box<dyn RodValidator>,
    check: F,
}

impl<F> fmt::Debug for RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone,
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
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone,
{
    pub fn new(schema: Box<dyn RodValidator>, check: F) -> Self {
        Self { schema, check }
    }
}

impl<F> RodValidator for RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // Validate Inner
        let val = self.schema.validate_with_context(ctx, input)?;

        // Check on RodValue (Zero Copy)
        if let Err(msg) = (self.check)(&val) {
            ctx.add_issue(
                RodIssueCode::Custom {
                    message: msg.clone(),
                    params: None,
                },
                msg,
            );
            return Err(());
        }
        Ok(val)
    }

    fn to_spec(&self) -> RodSpec {
        self.schema.to_spec()
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
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
    // CHANGED: Transformer now receives RodValue to allow lazy/zero-copy transformations
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone,
{
    schema: Box<dyn RodValidator>,
    transformer: F,
}

impl<F> fmt::Debug for RodTransform<F>
where
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone,
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
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone,
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
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        let val = self.schema.validate_with_context(ctx, input)?;

        // Transform value.
        // We assume transformer returns a value valid for 'a or 'static.
        Ok((self.transformer)(val))
    }

    fn to_spec(&self) -> RodSpec {
        self.schema.to_spec()
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
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
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    RodRefine::new(Box::new(validator), check)
}

pub fn transform<V, F>(validator: V, transformer: F) -> RodTransform<F>
where
    V: RodValidator + 'static,
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone + 'static,
{
    RodTransform::new(Box::new(validator), transformer)
}
