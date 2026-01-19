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
    fn validate(&self, input: &Value) -> RodResult<Value> {
        if input.is_null() {
            return Ok(Value::Null);
        }
        self.inner.validate(input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        // partial of nullable is nullable of partial
        Box::new(self.inner.deep_partial_boxed().nullable())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

// Extension trait to add .nullable() to all validators
pub trait NullableExtension: Sized + RodValidator {
    fn nullable(self) -> RodNullable
    where
        Self: 'static,
    {
        RodNullable::new(Box::new(self))
    }
}
// Implement for all types implementing RodValidator
impl<T: RodValidator + 'static> NullableExtension for T {}
