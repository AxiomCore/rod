use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodOptional {
    pub inner: Box<RodNode>,
}

impl RodOptional {
    pub fn new(inner: Box<RodNode>) -> Self {
        Self { inner }
    }
}

impl RodValidator for RodOptional {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Undefined {
            return Ok(RodValue::Null);
        }
        self.inner.validate_with_context(ctx, input)
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        let partial_inner = wrap_custom(self.inner.deep_partial_boxed());
        Box::new(RodOptional::new(Box::new(partial_inner)))
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

// Manual implementation to support trait erasure and break recursion
impl DynValidator for RodOptional {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &BoxedInput(input))
    }
    fn is_optional_dyn(&self) -> bool {
        true
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

pub trait OptionalExtension: Sized + RodValidator + IntoRodNode {
    fn optional(self) -> RodOptional
    where
        Self: 'static,
    {
        RodOptional::new(Box::new(self.into_node()))
    }
}
impl<T: RodValidator + IntoRodNode + 'static> OptionalExtension for T {}
