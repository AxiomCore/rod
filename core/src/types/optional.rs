use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;

#[derive(Debug, Clone)]
pub struct RodOptional {
    inner: Box<dyn RodValidator>,
}

impl RodOptional {
    pub fn new(inner: Box<dyn RodValidator>) -> Self {
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

        self.inner.validate_with_context(ctx, input)
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.inner.deep_partial_boxed().optional())
    }
    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub trait OptionalExtension: Sized + RodValidator {
    fn optional(self) -> RodOptional
    where
        Self: 'static,
    {
        RodOptional::new(Box::new(self))
    }
}
impl<T: RodValidator + 'static> OptionalExtension for T {}
