use crate::RodSpec;
use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone, Default)]
pub struct RodNumber {
    min: Option<f64>,
    max: Option<f64>,
    is_int: bool,
}

impl RodNumber {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn min(mut self, val: f64) -> Self {
        self.min = Some(val);
        self
    }
    pub fn max(mut self, val: f64) -> Self {
        self.max = Some(val);
        self
    }
    pub fn int(mut self) -> Self {
        self.is_int = true;
        self
    }
}

impl RodValidator for RodNumber {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Number {
            let maybe_u64 = input.as_u64();
            let maybe_i64 = input.as_i64();
            let maybe_f64 = input.as_f64();

            // 1. Integer Check
            if self.is_int {
                let is_integer = maybe_i64.is_some()
                    || maybe_u64.is_some()
                    || (maybe_f64.is_some() && maybe_f64.unwrap().fract() == 0.0);

                if !is_integer {
                    ctx.add_issue(
                        RodIssueCode::InvalidType {
                            expected: "integer".into(),
                            received: "float".into(),
                        },
                        "Expected integer, received float".into(),
                    );
                    return Err(());
                }
            }

            // 2. Bounds Check (Hybrid)
            if let Some(u_val) = maybe_u64 {
                if let Some(min) = self.min {
                    if min >= 0.0 && u_val < (min as u64) {
                        ctx.add_issue(
                            RodIssueCode::TooSmall {
                                minimum: min,
                                inclusive: true,
                                type_: "number".into(),
                            },
                            format!("Number too small"),
                        );
                    } else if min < 0.0 {
                        // u64 is always >= negative min
                    }
                }
                if let Some(max) = self.max {
                    if max >= 0.0 {
                        if u_val > (max as u64) {
                            ctx.add_issue(
                                RodIssueCode::TooBig {
                                    maximum: max,
                                    inclusive: true,
                                    type_: "number".into(),
                                },
                                format!("Number too big"),
                            );
                        }
                    }
                }
            } else if let Some(f_val) = maybe_f64 {
                if let Some(min) = self.min {
                    if f_val < min {
                        ctx.add_issue(
                            RodIssueCode::TooSmall {
                                minimum: min,
                                inclusive: true,
                                type_: "number".into(),
                            },
                            format!("Number too small"),
                        );
                    }
                }
                if let Some(max) = self.max {
                    if f_val > max {
                        ctx.add_issue(
                            RodIssueCode::TooBig {
                                maximum: max,
                                inclusive: true,
                                type_: "number".into(),
                            },
                            format!("Number too big"),
                        );
                    }
                }
            } else {
                ctx.add_issue(
                    RodIssueCode::Message {
                        message: "Internal number error".into(),
                    },
                    "Internal error".into(),
                );
                return Err(());
            }

            if ctx.has_issues() {
                return Err(());
            }

            return Ok(RodValue::Number(maybe_f64.unwrap_or(0.0)));
        }

        ctx.add_issue(
            RodIssueCode::InvalidType {
                expected: "number".into(),
                received: "unknown".into(),
            },
            "Expected number".into(),
        );
        Err(())
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Number {
            min: self.min,
            max: self.max,
            int: Some(self.is_int),
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

pub fn number() -> RodNumber {
    RodNumber::new()
}
