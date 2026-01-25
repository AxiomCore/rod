use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodEnum {
    pub values: Vec<String>,
}

impl RodEnum {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
}

impl RodValidator for RodEnum {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::String {
            if let Some(s) = input.as_str() {
                if self.values.iter().any(|v| v == s.as_ref()) {
                    return Ok(RodValue::String(s));
                }
            }
        }

        ctx.add_issue(
            RodIssueCode::InvalidEnumValue {
                expected: self.values.clone(),
                received: "".into(),
            },
            format!("Invalid enum value. Expected one of: {:?}", self.values),
        );
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodEnum {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

pub fn enum_type(values: Vec<&str>) -> RodEnum {
    RodEnum::new(values.iter().map(|s| s.to_string()).collect())
}
