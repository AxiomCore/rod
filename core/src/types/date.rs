use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct RodDate {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl RodDate {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn min(mut self, val: i64) -> Self {
        self.min = Some(val);
        self
    }
    pub fn max(mut self, val: i64) -> Self {
        self.max = Some(val);
        self
    }
}

impl RodValidator for RodDate {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        let (timestamp, val_to_return) = match input.get_type() {
            DataType::String => {
                let s = input.as_str().unwrap();
                match DateTime::parse_from_rfc3339(s.as_ref()) {
                    Ok(dt) => (
                        dt.with_timezone(&Utc).timestamp_millis(),
                        RodValue::String(s),
                    ),
                    Err(_) => {
                        ctx.add_issue(RodIssueCode::InvalidDate, "Invalid date format".into());
                        return Err(());
                    }
                }
            }
            DataType::Number => {
                let ms = if let Some(i) = input.as_i64() {
                    i
                } else if let Some(f) = input.as_f64() {
                    f as i64
                } else {
                    0
                };
                (ms, RodValue::Number(ms as f64))
            }
            _ => {
                ctx.add_issue(
                    RodIssueCode::InvalidType {
                        expected: "date".into(),
                        received: "unknown".into(),
                    },
                    "Expected date".into(),
                );
                return Err(());
            }
        };

        if let Some(min) = self.min {
            if timestamp < min {
                ctx.add_issue(
                    RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "date".into(),
                    },
                    "Date too small".into(),
                );
            }
        }

        if let Some(max) = self.max {
            if timestamp > max {
                ctx.add_issue(
                    RodIssueCode::TooBig {
                        maximum: max as f64,
                        inclusive: true,
                        type_: "date".into(),
                    },
                    "Date too big".into(),
                );
            }
        }

        if ctx.has_issues() {
            return Err(());
        }
        Ok(val_to_return)
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodDate {
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

pub fn date() -> RodDate {
    RodDate::new()
}
