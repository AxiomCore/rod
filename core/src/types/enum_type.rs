use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodEnum {
    values: Vec<String>,
}

impl RodEnum {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
}

impl RodValidator for RodEnum {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
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

    fn to_spec(&self) -> RodSpec {
        RodSpec::Enum {
            values: self.values.clone(),
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn enum_type(values: Vec<&str>) -> RodEnum {
    RodEnum::new(values.iter().map(|s| s.to_string()).collect())
}
