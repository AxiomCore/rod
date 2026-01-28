use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;

#[derive(Debug, Clone)]
pub struct RodNullable {
    inner: Box<dyn RodValidator>,
}

impl RodNullable {
    pub fn new(inner: Box<dyn RodValidator>) -> Self {
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
        self.inner.validate_with_context(ctx, input)
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Nullable(Box::new(self.inner.to_spec()))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.inner.deep_partial_boxed().nullable().optional())
    }
    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub trait NullableExtension: Sized + RodValidator {
    fn nullable(self) -> RodNullable
    where
        Self: 'static,
    {
        RodNullable::new(Box::new(self))
    }
}
impl<T: RodValidator + 'static> NullableExtension for T {}
