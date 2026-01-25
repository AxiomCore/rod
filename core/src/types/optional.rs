use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodOptional {
    inner: Box<RodNode>,
}

impl RodOptional {
    pub fn new(inner: Box<RodNode>) -> Self {
        Self { inner }
    }
}

impl RodValidator for RodOptional {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Undefined {
            return Ok(RodValue::Null);
        }

        // STATIC DISPATCH
        self.inner.validate_with_context(ctx, input)
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_inner = wrap_custom(self.inner.deep_partial_boxed());
        Box::new(RodOptional::new(Box::new(partial_inner)).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
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
