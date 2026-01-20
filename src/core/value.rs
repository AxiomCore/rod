use crate::core::input::{DataType, RodInput};
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum RodValue<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<RodValue<'a>>),
    Object(Vec<(Cow<'a, str>, RodValue<'a>)>),
    Json(Value),
}

impl<'a> RodValue<'a> {
    // --- Phase 1: Accessors ---

    pub fn as_str(&self) -> Option<&str> {
        match self {
            RodValue::String(s) => Some(s.as_ref()),
            RodValue::Json(Value::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            RodValue::Number(n) => Some(*n),
            RodValue::Json(Value::Number(n)) => n.as_f64(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RodValue::Boolean(b) => Some(*b),
            RodValue::Json(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            RodValue::Null => true,
            RodValue::Json(Value::Null) => true,
            _ => false,
        }
    }

    // --- End Phase 1 ---

    pub fn to_json(&self) -> Value {
        match self {
            RodValue::String(s) => Value::String(s.to_string()),
            RodValue::Number(n) => serde_json::Number::from_f64(*n)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            RodValue::Boolean(b) => Value::Bool(*b),
            RodValue::Null => Value::Null,
            RodValue::Array(arr) => Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            RodValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k.to_string(), v.to_json());
                }
                Value::Object(map)
            }
            RodValue::Json(v) => v.clone(),
        }
    }

    pub fn into_owned(self) -> RodValue<'static> {
        match self {
            RodValue::String(c) => RodValue::String(Cow::Owned(c.into_owned())),
            RodValue::Number(n) => RodValue::Number(n),
            RodValue::Boolean(b) => RodValue::Boolean(b),
            RodValue::Null => RodValue::Null,
            RodValue::Array(arr) => {
                RodValue::Array(arr.into_iter().map(|v| v.into_owned()).collect())
            }
            RodValue::Object(obj) => RodValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned()))
                    .collect(),
            ),
            RodValue::Json(v) => RodValue::Json(v),
        }
    }
}

#[derive(Debug)]
pub struct RodValueInput<'a>(pub &'a RodValue<'a>);

impl<'a> RodInput<'a> for RodValueInput<'a> {
    fn get_type(&self) -> DataType {
        match self.0 {
            RodValue::String(_) => DataType::String,
            RodValue::Number(_) => DataType::Number,
            RodValue::Boolean(_) => DataType::Boolean,
            RodValue::Null => DataType::Null,
            RodValue::Array(_) => DataType::Array,
            RodValue::Object(_) => DataType::Object,
            RodValue::Json(v) => match v {
                Value::String(_) => DataType::String,
                Value::Number(_) => DataType::Number,
                Value::Bool(_) => DataType::Boolean,
                Value::Null => DataType::Null,
                Value::Array(_) => DataType::Array,
                Value::Object(_) => DataType::Object,
            },
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        match self.0 {
            RodValue::String(s) => Some(s.clone()),
            RodValue::Json(Value::String(s)) => Some(Cow::Borrowed(s.as_str())),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self.0 {
            RodValue::Number(n) => Some(*n),
            RodValue::Json(Value::Number(n)) => n.as_f64(),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self.0 {
            RodValue::Number(n) => Some(*n as i64),
            RodValue::Json(Value::Number(n)) => n.as_i64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.0 {
            RodValue::Boolean(b) => Some(*b),
            RodValue::Json(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    fn get_key(&self, key: &str) -> Option<Box<dyn RodInput<'a> + '_>> {
        match self.0 {
            RodValue::Object(fields) => {
                for (k, v) in fields {
                    if k == key {
                        return Some(Box::new(RodValueInput(v)));
                    }
                }
                None
            }
            RodValue::Json(Value::Object(map)) => map.get(key).map(|v| {
                #[derive(Debug)]
                struct InnerJson<'b>(&'b Value);
                impl<'b> RodInput<'b> for InnerJson<'b> {
                    fn get_type(&self) -> DataType {
                        DataType::Unknown
                    }
                    fn as_str(&self) -> Option<Cow<'b, str>> {
                        self.0.as_str().map(Cow::Borrowed)
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
                    fn get_key(&self, k: &str) -> Option<Box<dyn RodInput<'b> + '_>> {
                        self.0
                            .get(k)
                            .map(|v| Box::new(InnerJson(v)) as Box<dyn RodInput<'b>>)
                    }
                    fn get_index(&self, i: usize) -> Option<Box<dyn RodInput<'b> + '_>> {
                        self.0
                            .get(i)
                            .map(|v| Box::new(InnerJson(v)) as Box<dyn RodInput<'b>>)
                    }
                    fn count(&self) -> Option<usize> {
                        None
                    }
                    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>> {
                        None
                    }
                    fn to_json(&self) -> Value {
                        self.0.clone()
                    }
                }
                Box::new(InnerJson(v)) as Box<dyn RodInput<'a>>
            }),
            _ => None,
        }
    }

    fn get_index(&self, index: usize) -> Option<Box<dyn RodInput<'a> + '_>> {
        match self.0 {
            RodValue::Array(arr) => arr
                .get(index)
                .map(|v| Box::new(RodValueInput(v)) as Box<dyn RodInput<'a>>),
            RodValue::Json(Value::Array(arr)) => arr.get(index).map(|v| {
                #[derive(Debug)]
                struct InnerJson<'b>(&'b Value);
                impl<'b> RodInput<'b> for InnerJson<'b> {
                    fn get_type(&self) -> DataType {
                        DataType::Unknown
                    }
                    fn as_str(&self) -> Option<Cow<'b, str>> {
                        self.0.as_str().map(Cow::Borrowed)
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
                    fn get_key(&self, k: &str) -> Option<Box<dyn RodInput<'b> + '_>> {
                        self.0
                            .get(k)
                            .map(|v| Box::new(InnerJson(v)) as Box<dyn RodInput<'b>>)
                    }
                    fn get_index(&self, i: usize) -> Option<Box<dyn RodInput<'b> + '_>> {
                        self.0
                            .get(i)
                            .map(|v| Box::new(InnerJson(v)) as Box<dyn RodInput<'b>>)
                    }
                    fn count(&self) -> Option<usize> {
                        None
                    }
                    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>> {
                        None
                    }
                    fn to_json(&self) -> Value {
                        self.0.clone()
                    }
                }
                Box::new(InnerJson(v)) as Box<dyn RodInput<'a>>
            }),
            _ => None,
        }
    }

    fn count(&self) -> Option<usize> {
        match self.0 {
            RodValue::Array(arr) => Some(arr.len()),
            RodValue::Object(obj) => Some(obj.len()),
            RodValue::Json(Value::Array(arr)) => Some(arr.len()),
            RodValue::Json(Value::Object(obj)) => Some(obj.len()),
            _ => None,
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>> {
        match self.0 {
            RodValue::Object(fields) => {
                let keys: Vec<String> = fields.iter().map(|(k, _)| k.to_string()).collect();
                Some(Box::new(keys.into_iter()))
            }
            RodValue::Json(Value::Object(map)) => Some(Box::new(map.keys().cloned())),
            _ => None,
        }
    }

    fn to_json(&self) -> Value {
        self.0.to_json()
    }
}
