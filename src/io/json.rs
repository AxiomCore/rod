// src/io/json.rs
use crate::core::input::{DataType, RodInput};
use serde_json::Value;

#[derive(Debug)]
pub struct JsonInput<'a>(pub &'a Value);

impl<'a> RodInput for JsonInput<'a> {
    fn get_type(&self) -> DataType {
        match self.0 {
            Value::String(_) => DataType::String,
            Value::Number(_) => DataType::Number,
            Value::Bool(_) => DataType::Boolean,
            Value::Null => DataType::Null,
            Value::Array(_) => DataType::Array,
            Value::Object(_) => DataType::Object,
        }
    }

    fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }

    fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    fn as_i64(&self) -> Option<i64> {
        self.0.as_i64()
    }

    fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    fn get_key(&self, key: &str) -> Option<Box<dyn RodInput + '_>> {
        self.0
            .get(key)
            .map(|v| Box::new(JsonInput(v)) as Box<dyn RodInput>)
    }

    fn get_index(&self, index: usize) -> Option<Box<dyn RodInput + '_>> {
        self.0
            .get(index)
            .map(|v| Box::new(JsonInput(v)) as Box<dyn RodInput>)
    }

    fn count(&self) -> Option<usize> {
        match self.0 {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>> {
        match self.0 {
            Value::Object(obj) => Some(Box::new(obj.keys().cloned())),
            _ => None,
        }
    }

    fn to_json(&self) -> Value {
        self.0.clone()
    }
}

pub fn wrap(v: &Value) -> JsonInput {
    JsonInput(v)
}
