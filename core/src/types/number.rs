use crate::core::input::{BoxedInput, DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone, Default)]
pub struct RodNumber {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub is_int: bool,
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
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        if input.get_type() == DataType::Number {
            let maybe_u64 = input.as_u64();
            let maybe_i64 = input.as_i64();
            let maybe_f64 = input.as_f64();

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
                        "Expected integer".into(),
                    );
                    return Err(());
                }
            }

            if let Some(u_val) = maybe_u64 {
                if let Some(min) = self.min {
                    if min >= 0.0 && u_val < (min as u64) {
                        ctx.add_issue(
                            RodIssueCode::TooSmall {
                                minimum: min,
                                inclusive: true,
                                type_: "number".into(),
                            },
                            "Number too small".into(),
                        );
                    }
                }
                if let Some(max) = self.max {
                    if max >= 0.0 && u_val > (max as u64) {
                        ctx.add_issue(
                            RodIssueCode::TooBig {
                                maximum: max,
                                inclusive: true,
                                type_: "number".into(),
                            },
                            "Number too big".into(),
                        );
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
                            "Number too small".into(),
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
                            "Number too big".into(),
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

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodNumber {
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

pub fn number() -> RodNumber {
    RodNumber::new()
}
