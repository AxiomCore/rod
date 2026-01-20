use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodError, RodResult};

#[derive(Debug, Clone)]
pub struct RodMap {
    key_type: Box<dyn RodValidator>,
    value_type: Box<dyn RodValidator>,
}

impl RodMap {
    pub fn new(key: Box<dyn RodValidator>, value: Box<dyn RodValidator>) -> Self {
        Self {
            key_type: key,
            value_type: value,
        }
    }
}

impl RodValidator for RodMap {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // Expecting Array of [Key, Value] tuples
        if input.get_type() != DataType::Array {
            return Err(RodError::new(
                "invalid_type",
                "Expected array of entries for Map",
            ));
        }

        let mut valid_entries = Vec::new();
        let mut issues = Vec::new();
        let len = input.count().unwrap_or(0);

        for i in 0..len {
            if let Some(entry_input) = input.get_index(i) {
                // entry_input must be an array of length 2
                // We access via reference &*entry_input
                if entry_input.get_type() == DataType::Array {
                    if entry_input.count() != Some(2) {
                        issues.push(crate::error::RodIssue {
                            details: crate::error::RodIssueCode::Custom {
                                message: "Map entry must be a [key, value] tuple".to_string(),
                                params: None,
                            },
                            message: "Map entry must be a [key, value] tuple".to_string(),
                            path: vec![i.to_string()],
                        });
                        continue;
                    }

                    let key_input = entry_input.get_index(0).unwrap();
                    let val_input = entry_input.get_index(1).unwrap();

                    // Validate Key & Value
                    let k_res = self.key_type.validate(key_input.as_ref());
                    let v_res = self.value_type.validate(val_input.as_ref());

                    match (k_res, v_res) {
                        (Ok(k), Ok(v)) => valid_entries.push(RodValue::Array(vec![k, v])), // Removed .into_owned()
                        (Err(mut e), _) => {
                            e.prepend_path(&format!("{}.key", i));
                            issues.extend(e.issues);
                        }
                        (_, Err(mut e)) => {
                            e.prepend_path(&format!("{}.value", i));
                            issues.extend(e.issues);
                        }
                    }
                } else {
                    issues.push(crate::error::RodIssue {
                        details: crate::error::RodIssueCode::InvalidType {
                            expected: "array".to_string(),
                            received: "unknown".to_string(),
                        },
                        message: "Map entries must be arrays".to_string(),
                        path: vec![i.to_string()],
                    });
                }
            }
        }

        if !issues.is_empty() {
            return Err(RodError { issues });
        }
        return Ok(RodValue::Array(valid_entries));
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_key = self.key_type.deep_partial_boxed();
        let partial_value = self.value_type.deep_partial_boxed();
        Box::new(RodMap::new(partial_key, partial_value).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn map(key: impl RodValidator + 'static, value: impl RodValidator + 'static) -> RodMap {
    RodMap::new(Box::new(key), Box::new(value))
}
