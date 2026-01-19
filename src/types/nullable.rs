use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::error::RodResult;
use serde_json::Value;

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
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        if input.get_type() == DataType::Null {
            return Ok(Value::Null);
        }
        self.inner.validate(input)
    }
    // ... clone/partial same ...
    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.inner.deep_partial_boxed().nullable().optional()) // Nullable + Optional
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
