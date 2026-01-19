use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::error::RodResult;
use serde_json::Value;

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
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        // If input is explicitly null, Zod treats it as invalid for optional unless .nullable()
        // But RodInput treats missing keys as... handled by parent object loop.
        // If we are here, the value exists (it might be Null or Undefined type).

        // Note: DataType::Undefined usually means JS `undefined`.
        if input.get_type() == DataType::Undefined {
            // Valid optional
            return Ok(Value::Null); // or Value::Null representing undefined? serde_json uses Null.
        }

        // Pass through
        self.inner.validate(input)
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
