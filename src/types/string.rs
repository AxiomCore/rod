use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodIssue, RodIssueCode, RodResult};
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^https?://").unwrap();
    static ref UUID_REGEX: Regex = Regex::new(r"^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$").unwrap();
    static ref CUID_REGEX: Regex = Regex::new(r"^c[^\s-]{8,}$").unwrap();
    static ref CUID2_REGEX: Regex = Regex::new(r"^[a-z][a-z0-9]*$").unwrap();
    static ref ULID_REGEX: Regex = Regex::new(r"^[0-9A-HJKMNP-TV-Z]{26}$").unwrap();
    static ref DATETIME_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?$").unwrap();
    static ref IP_V4_REGEX: Regex = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
    static ref IP_V6_REGEX: Regex = Regex::new(r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$").unwrap();
}

#[derive(Debug, Clone, Default)]
pub struct RodString {
    min: Option<usize>,
    max: Option<usize>,
    length: Option<usize>,
    is_email: bool,
    is_url: bool,
    is_uuid: bool,
    is_cuid: bool,
    is_cuid2: bool,
    is_ulid: bool,
    is_datetime: bool,
    is_ip: bool,
    starts_with: Option<String>,
    ends_with: Option<String>,
    includes: Option<String>,
    regex: Option<String>,
    should_trim: bool,
}

impl RodString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min(mut self, val: usize) -> Self {
        self.min = Some(val);
        self
    }
    pub fn max(mut self, val: usize) -> Self {
        self.max = Some(val);
        self
    }
    pub fn length(mut self, val: usize) -> Self {
        self.length = Some(val);
        self
    }
    pub fn email(mut self) -> Self {
        self.is_email = true;
        self
    }
    pub fn url(mut self) -> Self {
        self.is_url = true;
        self
    }
    pub fn uuid(mut self) -> Self {
        self.is_uuid = true;
        self
    }
    pub fn cuid(mut self) -> Self {
        self.is_cuid = true;
        self
    }
    pub fn cuid2(mut self) -> Self {
        self.is_cuid2 = true;
        self
    }
    pub fn ulid(mut self) -> Self {
        self.is_ulid = true;
        self
    }
    pub fn datetime(mut self) -> Self {
        self.is_datetime = true;
        self
    }
    pub fn ip(mut self) -> Self {
        self.is_ip = true;
        self
    }
    pub fn starts_with(mut self, val: &str) -> Self {
        self.starts_with = Some(val.to_string());
        self
    }
    pub fn ends_with(mut self, val: &str) -> Self {
        self.ends_with = Some(val.to_string());
        self
    }
    pub fn includes(mut self, val: &str) -> Self {
        self.includes = Some(val.to_string());
        self
    }
    pub fn regex(mut self, pattern: &str) -> Self {
        self.regex = Some(pattern.to_string());
        self
    }
    pub fn trim(mut self) -> Self {
        self.should_trim = true;
        self
    }
}

impl RodValidator for RodString {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // 1. Check Type
        if input.get_type() != DataType::String {
            return Err(RodError::with_issue(
                RodIssueCode::InvalidType {
                    expected: "string".to_string(),
                    received: format!("{:?}", input.get_type()).to_lowercase(),
                },
                format!("Expected string, received {:?}", input.get_type()),
            ));
        }

        // 2. Get Reference (Zero Copy)
        let s_cow = input
            .as_str()
            .ok_or_else(|| RodError::new("internal", "Failed to read string from input"))?;

        // 3. Handle Transforms (trim)
        // If we trim a Borrowed string, we get a sub-slice, which is also Borrowed.
        // This avoids allocation completely.
        let val_cow = if self.should_trim {
            match s_cow {
                Cow::Borrowed(s) => Cow::Borrowed(s.trim()),
                Cow::Owned(s) => Cow::Owned(s.trim().to_string()),
            }
        } else {
            s_cow
        };

        // We check against the (possibly trimmed) string
        let check_str = val_cow.as_ref();

        let mut issues = Vec::new();

        // Length Checks
        if let Some(min) = self.min {
            if check_str.len() < min {
                issues.push(RodIssue {
                    details: RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "string".to_string(),
                    },
                    message: format!("String must contain at least {} character(s)", min),
                    path: vec![],
                });
            }
        }
        if let Some(max) = self.max {
            if check_str.len() > max {
                issues.push(RodIssue {
                    details: RodIssueCode::TooBig {
                        maximum: max as f64,
                        inclusive: true,
                        type_: "string".to_string(),
                    },
                    message: format!("String must contain at most {} character(s)", max),
                    path: vec![],
                });
            }
        }
        if let Some(len) = self.length {
            if check_str.len() != len {
                issues.push(RodIssue {
                    details: RodIssueCode::InvalidString {
                        validation: "length".to_string(),
                    },
                    message: format!("String must contain exactly {} character(s)", len),
                    path: vec![],
                });
            }
        }

        // Regex / Format Checks
        if self.is_email && !EMAIL_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "email".to_string(),
                },
                message: "Invalid email".to_string(),
                path: vec![],
            });
        }
        if self.is_url && !URL_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "url".to_string(),
                },
                message: "Invalid url".to_string(),
                path: vec![],
            });
        }
        if self.is_uuid && !UUID_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "uuid".to_string(),
                },
                message: "Invalid UUID".to_string(),
                path: vec![],
            });
        }
        if self.is_cuid && !CUID_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "cuid".to_string(),
                },
                message: "Invalid CUID".to_string(),
                path: vec![],
            });
        }
        if self.is_cuid2 && !CUID2_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "cuid2".to_string(),
                },
                message: "Invalid CUID2".to_string(),
                path: vec![],
            });
        }
        if self.is_ulid && !ULID_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "ulid".to_string(),
                },
                message: "Invalid ULID".to_string(),
                path: vec![],
            });
        }
        if self.is_datetime && !DATETIME_REGEX.is_match(check_str) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidDate,
                message: "Invalid datetime".to_string(),
                path: vec![],
            });
        }
        if self.is_ip && !(IP_V4_REGEX.is_match(check_str) || IP_V6_REGEX.is_match(check_str)) {
            issues.push(RodIssue {
                details: RodIssueCode::InvalidString {
                    validation: "ip".to_string(),
                },
                message: "Invalid IP address".to_string(),
                path: vec![],
            });
        }

        // Content Checks
        if let Some(start) = &self.starts_with {
            if !check_str.starts_with(start) {
                issues.push(RodIssue {
                    details: RodIssueCode::InvalidString {
                        validation: "starts_with".to_string(),
                    },
                    message: format!("Must start with \"{}\"", start),
                    path: vec![],
                });
            }
        }
        if let Some(end) = &self.ends_with {
            if !check_str.ends_with(end) {
                issues.push(RodIssue {
                    details: RodIssueCode::InvalidString {
                        validation: "ends_with".to_string(),
                    },
                    message: format!("Must end with \"{}\"", end),
                    path: vec![],
                });
            }
        }
        if let Some(inc) = &self.includes {
            if !check_str.contains(inc) {
                issues.push(RodIssue {
                    details: RodIssueCode::InvalidString {
                        validation: "includes".to_string(),
                    },
                    message: format!("Must include \"{}\"", inc),
                    path: vec![],
                });
            }
        }
        if let Some(pattern) = &self.regex {
            if let Ok(re) = Regex::new(pattern) {
                if !re.is_match(check_str) {
                    issues.push(RodIssue {
                        details: RodIssueCode::InvalidString {
                            validation: "regex".to_string(),
                        },
                        message: "Invalid".to_string(),
                        path: vec![],
                    });
                }
            }
        }

        if !issues.is_empty() {
            return Err(RodError { issues });
        }

        // 3. Return RodValue (Borrowed if possible)
        Ok(RodValue::String(val_cow))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        Box::new(self.clone().optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn string() -> RodString {
    RodString::new()
}
