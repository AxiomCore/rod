use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};

#[derive(Debug, Clone, Default)]
pub struct RodBoolean;

impl RodValidator for RodBoolean {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        if input.get_type() == DataType::Boolean {
            let val = input.as_bool().unwrap_or(false);
            return Ok(RodValue::Boolean(val));
        }
        Err(RodError::new("invalid_type", "Expected boolean"))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn boolean() -> RodBoolean {
    RodBoolean
}
