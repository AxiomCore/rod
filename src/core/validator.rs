use crate::core::input::RodInput; // Import the trait
use crate::error::RodResult;
use serde_json::Value;
use std::fmt::Debug;

pub trait RodValidator: Send + Sync + Debug {
    // CHANGE: Input is now &dyn RodInput
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value>;

    fn is_optional(&self) -> bool {
        false
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator>;

    fn clone_box(&self) -> Box<dyn RodValidator>;
}

impl Clone for Box<dyn RodValidator> {
    fn clone(&self) -> Box<dyn RodValidator> {
        self.clone_box()
    }
}

impl RodValidator for Box<dyn RodValidator> {
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        (**self).validate(input)
    }

    fn is_optional(&self) -> bool {
        (**self).is_optional()
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        (**self).deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        (**self).clone_box()
    }
}
