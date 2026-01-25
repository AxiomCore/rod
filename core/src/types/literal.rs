use crate::core::input::{BoxedInput, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodLiteral {
    pub expected: Value,
}

impl RodLiteral {
    pub fn new(expected: Value) -> Self {
        Self { expected }
    }
}

impl RodValidator for RodLiteral {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        let is_match = match &self.expected {
            Value::String(s) => input
                .as_str()
                .map_or(false, |input_str| input_str.as_ref() == s),
            Value::Number(n) => input.as_f64().map_or(false, |input_num| {
                n.as_f64().map_or(false, |expected_f64| {
                    (input_num - expected_f64).abs() < f64::EPSILON
                })
            }),
            Value::Bool(b) => input.as_bool().map_or(false, |input_bool| input_bool == *b),
            Value::Null => input.get_type() == crate::core::input::DataType::Null,
            _ => input.to_json() == self.expected,
        };

        if is_match {
            match &self.expected {
                Value::String(_) => return Ok(RodValue::String(input.as_str().unwrap())),
                Value::Number(_) => return Ok(RodValue::Number(input.as_f64().unwrap())),
                Value::Bool(b) => return Ok(RodValue::Boolean(*b)),
                Value::Null => return Ok(RodValue::Null),
                _ => return Ok(RodValue::Json(input.to_json())),
            }
        }

        ctx.add_issue(
            RodIssueCode::InvalidLiteral {
                expected: format!("{}", self.expected),
            },
            format!("Invalid literal value, expected {}", self.expected),
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

impl DynValidator for RodLiteral {
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

pub fn literal<T: Into<Value>>(val: T) -> RodLiteral {
    RodLiteral::new(val.into())
}
