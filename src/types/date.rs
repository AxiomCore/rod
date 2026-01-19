use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodIssue, RodIssueCode, RodResult};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RodDate {
    min: Option<i64>, // Timestamp in milliseconds
    max: Option<i64>, // Timestamp in milliseconds
}

impl Default for RodDate {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

impl RodDate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum date (inclusive) using a timestamp (ms)
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
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // We expect an ISO string by default for JSON compatibility
        if input.get_type() != DataType::String {
            return Err(RodError::new("invalid_type", "Expected date string"));
        }

        let date_str = input
            .as_str()
            .ok_or_else(|| RodError::new("internal", "Failed to read string"))?;

        // Parse using chrono
        let dt = match DateTime::parse_from_rfc3339(date_str.as_ref()) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                return Err(RodError::new("invalid_date", "Invalid date format"));
            }
        };

        let timestamp = dt.timestamp_millis();
        let mut issues = Vec::new();

        if let Some(min) = self.min {
            if timestamp < min {
                issues.push(RodIssue {
                    details: RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "date".to_string(),
                    },
                    message: format!("Date must be greater than or equal to timestamp {}", min),
                    path: vec![],
                });
            }
        }

        if let Some(max) = self.max {
            if timestamp > max {
                issues.push(RodIssue {
                    details: RodIssueCode::TooBig {
                        maximum: max as f64,
                        inclusive: true,
                        type_: "date".to_string(),
                    },
                    message: format!("Date must be smaller than or equal to timestamp {}", max),
                    path: vec![],
                });
            }
        }

        if !issues.is_empty() {
            return Err(RodError { issues });
        }

        // Return RodValue::String (borrowed)
        Ok(RodValue::String(date_str))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn date() -> RodDate {
    RodDate::new()
}
