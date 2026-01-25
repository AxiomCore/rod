use crate::core::input::RodInput;
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode};
use std::fmt;

// --- Refine ---

#[derive(Clone)]
pub struct RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone,
{
    pub schema: Box<RodNode>,
    pub check: F,
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
    pub fn new(schema: RodNode, check: F) -> Self {
        Self {
            schema: Box::new(schema),
            check,
        }
    }
}

impl<F> RodValidator for RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        // MONOMORPHIZED CALL: Specialized validation of the inner schema
        let val = self.schema.validate_with_context(ctx, input)?;

        // Custom Refinement Logic
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

// --- Transform ---

#[derive(Clone)]
pub struct RodTransform<F>
where
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone,
{
    pub schema: Box<RodNode>,
    pub transformer: F,
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
    pub fn new(schema: RodNode, transformer: F) -> Self {
        Self {
            schema: Box::new(schema),
            transformer,
        }
    }
}

impl<F> RodValidator for RodTransform<F>
where
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        // MONOMORPHIZED CALL
        let val = self.schema.validate_with_context(ctx, input)?;

        // Transformation Logic
        Ok((self.transformer)(val))
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

// --- Helpers ---

pub fn refine<V, F>(validator: V, check: F) -> RodNode
where
    V: IntoRodNode + 'static,
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    RodNode::Custom(Box::new(RodRefine::new(validator.into_node(), check)))
}

pub fn transform<V, F>(validator: V, transformer: F) -> RodNode
where
    V: IntoRodNode + 'static,
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone + 'static,
{
    RodNode::Custom(Box::new(RodTransform::new(
        validator.into_node(),
        transformer,
    )))
}

impl<F> DynValidator for RodRefine<F>
where
    F: for<'v> Fn(&RodValue<'v>) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &crate::core::input::BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

impl<F> DynValidator for RodTransform<F>
where
    F: Fn(RodValue) -> RodValue + Send + Sync + Clone + 'static,
{
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &crate::core::input::BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}
