use crate::RodSpec;
use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RodLiteral {
    expected: Value,
}

impl RodLiteral {
    pub fn new(expected: Value) -> Self {
        Self { expected }
    }
}

impl RodValidator for RodLiteral {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // Optimization: Compare input against expected primitives without allocating full JSON
        let is_match = match &self.expected {
            Value::String(s) => {
                if let Some(input_str) = input.as_str() {
                    input_str.as_ref() == s
                } else {
                    false
                }
            }
            Value::Number(n) => {
                if let Some(input_num) = input.as_f64() {
                    if let Some(expected_f64) = n.as_f64() {
                        (input_num - expected_f64).abs() < f64::EPSILON
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Value::Bool(b) => {
                if let Some(input_bool) = input.as_bool() {
                    input_bool == *b
                } else {
                    false
                }
            }
            Value::Null => input.get_type() == crate::core::input::DataType::Null,
            _ => {
                let val = input.to_json();
                val == self.expected
            }
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

    fn to_spec(&self) -> RodSpec {
        RodSpec::Literal {
            value: self.expected.clone(),
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

pub fn literal<T: Into<Value>>(val: T) -> RodLiteral {
    RodLiteral::new(val.into())
}
