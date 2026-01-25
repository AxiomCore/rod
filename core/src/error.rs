use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "code_type", rename_all = "snake_case")]
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
    TooDeep {
        depth: usize,
    },
    Custom {
        message: String,
        params: Option<serde_json::Map<String, Value>>,
    },
    Message {
        message: String,
    },
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct RodIssue {
    pub details: RodIssueCode,
    pub path: Vec<String>,
    pub message: String,
}

#[derive(Debug, Error, Serialize)]
#[error("Validation failed with {issues:?}")]
pub struct RodError {
    pub issues: Vec<RodIssue>,
}

impl RodError {
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

        Self {
            issues: vec![RodIssue {
                details: code,
                path: vec![],
                message: message.to_string(),
            }],
        }
    }

    // Helper to create a new error with a specific issue code
    pub fn with_issue(code: RodIssueCode, message: String) -> Self {
        Self {
            issues: vec![RodIssue {
                details: code,
                path: vec![],
                message,
            }],
        }
    }
}

pub type RodResult<T> = Result<T, RodError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationMode {
    Strict,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment<'a> {
    Key(Cow<'a, str>),
    Index(usize),
}

impl<'a> fmt::Display for PathSegment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Key(k) => write!(f, "{}", k),
            PathSegment::Index(i) => write!(f, "{}", i),
        }
    }
}

const MAX_DEPTH: usize = 256;

pub struct ValidationContext<'a> {
    pub issues: Vec<RodIssue>,
    pub path: Vec<PathSegment<'a>>,
    pub mode: ValidationMode,
    pub depth: usize,
}

impl<'a> ValidationContext<'a> {
    pub fn new() -> Self {
        Self {
            issues: Vec::with_capacity(4),
            path: Vec::with_capacity(16),
            mode: ValidationMode::All,
            depth: 0,
        }
    }

    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Fork a context for trial validation (e.g. Union options).
    /// Critical: Inherits depth to prevent recursion attacks.
    pub fn fork(&self) -> Self {
        Self {
            issues: Vec::with_capacity(4),
            path: self.path.clone(),
            mode: self.mode,
            depth: self.depth,
        }
    }

    fn enter_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        if self.depth >= MAX_DEPTH {
            self.add_issue(
                RodIssueCode::TooDeep { depth: self.depth },
                "Recursion limit exceeded".into(),
            );
        }

        self.depth += 1;
        let res = f(self);
        self.depth -= 1;
        res
    }

    pub fn with_path<F, R>(&mut self, segment: &'a str, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.path.push(PathSegment::Key(Cow::Borrowed(segment)));
        let result = self.enter_scope(f);
        self.path.pop();
        result
    }

    pub fn with_index<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.path.push(PathSegment::Index(index));
        let result = self.enter_scope(f);
        self.path.pop();
        result
    }

    pub fn with_owned_path<F, R>(&mut self, segment: String, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.path.push(PathSegment::Key(Cow::Owned(segment)));
        let result = self.enter_scope(f);
        self.path.pop();
        result
    }

    pub fn add_issue(&mut self, code: RodIssueCode, message: String) {
        // Flatten path only on error
        let flattened_path = self.path.iter().map(|p| p.to_string()).collect();

        self.issues.push(RodIssue {
            details: code,
            path: flattened_path,
            message,
        });
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn should_abort(&self) -> bool {
        (self.mode == ValidationMode::Strict && self.has_issues()) || self.depth > MAX_DEPTH
    }
}
