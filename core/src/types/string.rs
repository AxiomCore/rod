use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::net::IpAddr;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    static ref DATETIME_REGEX: Regex =
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?$").unwrap();
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

fn is_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return false;
    }
    for (i, &b) in bytes.iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            continue;
        }
        if !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn is_cuid(s: &str) -> bool {
    s.starts_with('c') && s.len() >= 8
}

impl RodValidator for RodString {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // 1. Basic Type Check
        if input.get_type() != DataType::String {
            ctx.add_issue(
                RodIssueCode::InvalidType {
                    expected: "string".into(),
                    received: "unknown".into(),
                },
                "Expected string".into(),
            );
            return Err(());
        }

        // 2. --- OPTIMIZATION: Length Fast-Path ---
        // We use as_str_len() which (in WASM) calls JS .length without copying bytes.
        let check_len = input.as_str_len().unwrap_or(0);

        if let Some(min) = self.min {
            if check_len < min {
                ctx.add_issue(
                    RodIssueCode::TooSmall {
                        minimum: min as f64,
                        inclusive: true,
                        type_: "string".into(),
                    },
                    format!("String too short"),
                );
            }
        }
        if let Some(max) = self.max {
            if check_len > max {
                ctx.add_issue(
                    RodIssueCode::TooBig {
                        maximum: max as f64,
                        inclusive: true,
                        type_: "string".into(),
                    },
                    format!("String too long"),
                );
            }
        }
        if let Some(len) = self.length {
            if check_len != len {
                ctx.add_issue(
                    RodIssueCode::InvalidString {
                        validation: "length".into(),
                    },
                    format!("Invalid length"),
                );
            }
        }

        // 3. Determine if we need the actual string content
        let needs_content = self.is_email
            || self.is_url
            || self.is_uuid
            || self.is_cuid
            || self.is_cuid2
            || self.is_ulid
            || self.is_datetime
            || self.is_ip
            || self.starts_with.is_some()
            || self.ends_with.is_some()
            || self.includes.is_some()
            || self.regex.is_some()
            || self.should_trim;

        // If no content checks are needed and we have no issues, return Lazy cursor (Zero Copy)
        if !needs_content {
            if ctx.has_issues() {
                return Err(());
            }
            return Ok(RodValue::Lazy(input.clone_box()));
        }

        // 4. --- SLOW PATH: Content Validation ---
        // This triggers the actual JS -> Rust string copy/allocation
        let s_cow = input.as_str().ok_or_else(|| {
            ctx.add_issue(
                RodIssueCode::Message {
                    message: "Internal string error".into(),
                },
                "Internal error".into(),
            );
        })?;

        let val_cow = if self.should_trim {
            match s_cow {
                Cow::Borrowed(s) => Cow::Borrowed(s.trim()),
                Cow::Owned(s) => Cow::Owned(s.trim().to_string()),
            }
        } else {
            s_cow
        };

        let check_str = val_cow.as_ref();

        // Perform specialized checks
        if self.is_email && !EMAIL_REGEX.is_match(check_str) {
            ctx.add_issue(
                RodIssueCode::InvalidString {
                    validation: "email".into(),
                },
                "Invalid email".into(),
            );
        }
        if self.is_url && !(check_str.starts_with("http://") || check_str.starts_with("https://")) {
            ctx.add_issue(
                RodIssueCode::InvalidString {
                    validation: "url".into(),
                },
                "Invalid url".into(),
            );
        }
        if self.is_uuid && !is_uuid(check_str) {
            ctx.add_issue(
                RodIssueCode::InvalidString {
                    validation: "uuid".into(),
                },
                "Invalid UUID".into(),
            );
        }
        if self.is_cuid && !is_cuid(check_str) {
            ctx.add_issue(
                RodIssueCode::InvalidString {
                    validation: "cuid".into(),
                },
                "Invalid CUID".into(),
            );
        }
        if self.is_ip && check_str.parse::<IpAddr>().is_err() {
            ctx.add_issue(
                RodIssueCode::InvalidString {
                    validation: "ip".into(),
                },
                "Invalid IP address".into(),
            );
        }
        if self.is_datetime && !DATETIME_REGEX.is_match(check_str) {
            ctx.add_issue(RodIssueCode::InvalidDate, "Invalid datetime".into());
        }
        if let Some(start) = &self.starts_with {
            if !check_str.starts_with(start) {
                ctx.add_issue(
                    RodIssueCode::InvalidString {
                        validation: "starts_with".into(),
                    },
                    "Invalid start".into(),
                );
            }
        }
        if let Some(end) = &self.ends_with {
            if !check_str.ends_with(end) {
                ctx.add_issue(
                    RodIssueCode::InvalidString {
                        validation: "ends_with".into(),
                    },
                    "Invalid end".into(),
                );
            }
        }
        if let Some(inc) = &self.includes {
            if !check_str.contains(inc) {
                ctx.add_issue(
                    RodIssueCode::InvalidString {
                        validation: "includes".into(),
                    },
                    "Missing substring".into(),
                );
            }
        }
        if let Some(pattern) = &self.regex {
            if let Ok(re) = Regex::new(pattern) {
                if !re.is_match(check_str) {
                    ctx.add_issue(
                        RodIssueCode::InvalidString {
                            validation: "regex".into(),
                        },
                        "Regex mismatch".into(),
                    );
                }
            }
        }

        if ctx.has_issues() {
            return Err(());
        }

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
