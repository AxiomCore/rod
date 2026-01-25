use crate::core::input::{DataType, RodInput};
use crate::core::value::RodValue;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug)]
pub struct JsonInput<'a>(pub &'a Value);

impl<'a> RodInput<'a> for JsonInput<'a> {
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

    fn as_str(&self) -> Option<Cow<'a, str>> {
        self.0.as_str().map(Cow::Borrowed)
    }

    fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    fn as_i64(&self) -> Option<i64> {
        self.0.as_i64()
    }

    fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }

    fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        let val = self.0.get(key)?;
        let wrapper = JsonInput(val);
        Some(f(&wrapper))
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        let val = self.0.get(index)?;
        let wrapper = JsonInput(val);
        Some(f(&wrapper))
    }

    fn count(&self) -> Option<usize> {
        match self.0 {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>> {
        match self.0 {
            Value::Object(obj) => Some(Box::new(obj.keys().map(|k| Cow::Borrowed(k.as_str())))),
            _ => None,
        }
    }

    fn to_json(&self) -> Value {
        self.0.clone()
    }

    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(JsonInput(self.0))
    }
}

pub fn wrap(v: &Value) -> JsonInput {
    JsonInput(v)
}
