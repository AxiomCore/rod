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
    fn validate(&self, input: &Value) -> RodResult<Value> {
        // If it's explicitly null, Zod optional usually treats it as an error
        // (unless .nullable().optional()).
        // But if the key was present and value was null, validate() is called.
        // If the key was missing, RodObject won't call this (due to is_optional check).

        // However, if we pass explicit `Value::Null` to an optional,
        // Zod treats `optional` as `T | undefined`. It does NOT allow `null`.
        // So we just pass through to inner.
        self.inner.validate(input)
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        // deep partial of optional is optional of deep partial inner
        Box::new(self.inner.deep_partial_boxed().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

// Extension trait
pub trait OptionalExtension: Sized + RodValidator {
    fn optional(self) -> RodOptional
    where
        Self: 'static,
    {
        RodOptional::new(Box::new(self))
    }
}
impl<T: RodValidator + 'static> OptionalExtension for T {}
