use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone, Default)]
pub struct RodBoolean;

impl RodValidator for RodBoolean {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Boolean {
            let val = input.as_bool().unwrap_or(false);
            return Ok(RodValue::Boolean(val));
        }
        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "boolean".into(),
                received: "unknown".into(),
            },
            "Expected boolean".into(),
        );
        Err(())
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Boolean
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
