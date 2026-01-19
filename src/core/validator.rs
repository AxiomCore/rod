use crate::core::input::RodInput;
use crate::core::value::RodValue;
use crate::error::RodResult;
use std::fmt::Debug;

pub trait RodValidator: Send + Sync + Debug {
    // input is a reference to a trait object that handles data of lifetime 'a.
    // The reference itself (&) can be short-lived.
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>>;

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
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
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
