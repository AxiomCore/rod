use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum RodIssueCode {
    InvalidType {
        expected: String,
        received: String,
    },
    InvalidLiteral {
        expected: String,
    },
    UnrecognizedKeys {
        keys: Vec<String>,
    },
    InvalidUnion,
    InvalidUnionDiscriminator {
        expected: Vec<String>,
    },
    InvalidEnumValue {
        expected: Vec<String>,
        received: String,
    },
    InvalidArguments,
    InvalidReturnType,
    InvalidDate,
    InvalidString {
        validation: String, // email, url, regex, etc.
    },
    TooSmall {
        minimum: f64, // using f64 to cover number/date/length
        inclusive: bool,
        #[serde(rename = "type")]
        type_: String, // array, string, number, set
    },
    TooBig {
        maximum: f64,
        inclusive: bool,
        #[serde(rename = "type")]
        type_: String,
    },
    Custom {
        message: String,
        params: Option<serde_json::Map<String, Value>>,
    },
    // Generic fallback for now
    Message {
        message: String,
    },
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct RodIssue {
    #[serde(flatten)]
    pub details: RodIssueCode,
    pub path: Vec<String>,
    pub message: String, // Calculated message
}

#[derive(Debug, Error, Serialize)]
#[error("Validation failed with {issues:?}")]
pub struct RodError {
    pub issues: Vec<RodIssue>,
}

impl RodError {
    // Helper to create a new error with a specific code
    pub fn with_issue(code: RodIssueCode, message: String) -> Self {
        Self {
            issues: vec![RodIssue {
                details: code,
                path: vec![],
                message,
            }],
        }
    }

    // Legacy helper for migration (maps to Custom or Message)
    pub fn new(code_str: &str, message: &str) -> Self {
        let code = match code_str {
            "custom_error" => RodIssueCode::Custom {
                message: message.to_string(),
                params: None,
            },
            _ => RodIssueCode::Message {
                message: message.to_string(),
            },
        };

        Self::with_issue(code, message.to_string())
    }

    pub fn prepend_path(&mut self, key: &str) {
        for issue in &mut self.issues {
            issue.path.insert(0, key.to_string());
        }
    }
}

pub type RodResult<T> = Result<T, RodError>;
