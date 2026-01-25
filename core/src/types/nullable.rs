use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodNullable {
    inner: Box<RodNode>,
}

impl RodNullable {
    pub fn new(inner: Box<RodNode>) -> Self {
        Self { inner }
    }
}

impl RodValidator for RodNullable {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Null {
            return Ok(RodValue::Null);
        }
        // STATIC DISPATCH
        self.inner.validate_with_context(ctx, input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_inner = wrap_custom(self.inner.deep_partial_boxed());
        Box::new(
            RodNullable::new(Box::new(partial_inner))
                .nullable()
                .optional(),
        )
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub trait NullableExtension: Sized + RodValidator + IntoRodNode {
    fn nullable(self) -> RodNullable
    where
        Self: 'static,
    {
        RodNullable::new(Box::new(self.into_node()))
    }
}
impl<T: RodValidator + IntoRodNode + 'static> NullableExtension for T {}
