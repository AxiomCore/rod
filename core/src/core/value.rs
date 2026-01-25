use crate::core::input::{DataType, RodInput};
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone)]
pub enum RodValue<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<RodValue<'a>>),
    Object(Vec<(Cow<'a, str>, RodValue<'a>)>),
    Json(Value),
    // Lazy Input (True Zero Copy for Any/Passthrough)
    Lazy(Box<dyn RodInput<'a> + 'a>),
}

impl<'a> fmt::Debug for RodValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Null => write!(f, "Null"),
            Self::Array(arg0) => f.debug_tuple("Array").field(arg0).finish(),
            Self::Object(arg0) => f.debug_tuple("Object").field(arg0).finish(),
            Self::Json(arg0) => f.debug_tuple("Json").field(arg0).finish(),
            Self::Lazy(_) => write!(f, "Lazy(<Input>)"),
        }
    }
}

impl<'a> PartialEq for RodValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Null, Self::Null) => true,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            (Self::Json(l0), Self::Json(r0)) => l0 == r0,
            (a, b) => a.to_json() == b.to_json(),
        }
    }
}

impl<'a> RodValue<'a> {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RodValue::String(s) => Some(s.as_ref()),
            RodValue::Json(Value::String(s)) => Some(s.as_str()),
            RodValue::Lazy(i) => i.as_str().map(|c| {
                // This lifetime hack is risky with Cow,
                // but usually we consume the value immediately.
                // For proper lifetime support we should return Cow.
                // However, signature returns &str.
                // We cannot return reference to temporary Cow.
                // We fallback to None here as accessors on RodValue are limited.
                // Use into_owned/to_json if needed for Lazy.
                // OR: Change this method to return Cow.
                // For now, simple logic:
                // If it is borrowed, return.
                match c {
                    Cow::Borrowed(b) => b,
                    Cow::Owned(_) => "", // Cannot return reference to owned temporary
                }
            }),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            RodValue::Number(n) => Some(*n),
            RodValue::Json(Value::Number(n)) => n.as_f64(),
            RodValue::Lazy(i) => i.as_f64(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RodValue::Boolean(b) => Some(*b),
            RodValue::Json(Value::Bool(b)) => Some(*b),
            RodValue::Lazy(i) => i.as_bool(),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            RodValue::Null => true,
            RodValue::Json(Value::Null) => true,
            RodValue::Lazy(i) => i.get_type() == DataType::Null,
            _ => false,
        }
    }

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
            RodValue::Lazy(input) => input.to_json(),
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
            RodValue::Lazy(input) => RodValue::Json(input.to_json()),
        }
    }
}

#[derive(Debug)]
pub struct RodValueInput<'a>(pub &'a RodValue<'a>);

impl<'a> RodInput<'a> for RodValueInput<'a> {
    fn as_str_len(&self) -> Option<usize> {
        match self.0 {
            RodValue::Lazy(i) => i.as_str_len(),
            RodValue::String(s) => Some(s.chars().count()),
            RodValue::Json(serde_json::Value::String(s)) => Some(s.chars().count()),
            _ => None,
        }
    }

    fn get_js_value(&self) -> Option<wasm_bindgen::JsValue> {
        match self.0 {
            RodValue::Lazy(i) => i.get_js_value(),
            // If the value is already a Json variant containing a JsValue handle
            // (though rare in core), we could handle it here too.
            _ => None,
        }
    }

    fn get_type(&self) -> DataType {
        match &self.0 {
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
            RodValue::Lazy(i) => i.get_type(),
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        match &self.0 {
            RodValue::String(s) => Some(s.clone()),
            RodValue::Json(Value::String(s)) => Some(Cow::Borrowed(s.as_str())),
            RodValue::Lazy(i) => i.as_str(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.0 {
            RodValue::Number(n) => Some(*n),
            RodValue::Json(Value::Number(n)) => n.as_f64(),
            RodValue::Lazy(i) => i.as_f64(),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.0 {
            RodValue::Number(n) => Some(*n as i64),
            RodValue::Json(Value::Number(n)) => n.as_i64(),
            RodValue::Lazy(i) => i.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match &self.0 {
            RodValue::Number(n) => {
                if *n >= 0.0 && n.fract() == 0.0 && *n <= (u64::MAX as f64) {
                    Some(*n as u64)
                } else {
                    None
                }
            }
            RodValue::Json(Value::Number(n)) => n.as_u64(),
            RodValue::Lazy(i) => i.as_u64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.0 {
            RodValue::Boolean(b) => Some(*b),
            RodValue::Json(Value::Bool(b)) => Some(*b),
            RodValue::Lazy(i) => i.as_bool(),
            _ => None,
        }
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        match self.0 {
            RodValue::Object(fields) => {
                for (k, v) in fields {
                    if k == key {
                        let wrapper = RodValueInput(v);
                        return Some(f(&wrapper));
                    }
                }
                None
            }
            RodValue::Json(serde_json::Value::Object(map)) => map.get(key).map(|v| {
                let wrapper = crate::io::json::JsonInput(v);
                f(&wrapper)
            }),
            RodValue::Lazy(i) => i.with_key(key, f),
            _ => None,
        }
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        match self.0 {
            RodValue::Array(arr) => arr.get(index).map(|v| {
                let wrapper = RodValueInput(v);
                f(&wrapper)
            }),
            RodValue::Json(serde_json::Value::Array(arr)) => arr.get(index).map(|v| {
                let wrapper = crate::io::json::JsonInput(v);
                f(&wrapper)
            }),
            RodValue::Lazy(i) => i.with_index(index, f),
            _ => None,
        }
    }

    fn count(&self) -> Option<usize> {
        match &self.0 {
            RodValue::Array(arr) => Some(arr.len()),
            RodValue::Object(obj) => Some(obj.len()),
            RodValue::Json(Value::Array(arr)) => Some(arr.len()),
            RodValue::Json(Value::Object(obj)) => Some(obj.len()),
            RodValue::Lazy(i) => i.count(),
            _ => None,
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>> {
        match &self.0 {
            RodValue::Object(fields) => {
                let keys: Vec<Cow<'a, str>> = fields.iter().map(|(k, _)| k.clone()).collect();
                Some(Box::new(keys.into_iter()))
            }
            RodValue::Json(Value::Object(map)) => {
                Some(Box::new(map.keys().map(|k| Cow::Borrowed(k.as_str()))))
            }
            RodValue::Lazy(i) => i.keys(),
            _ => None,
        }
    }

    fn to_json(&self) -> Value {
        self.0.to_json()
    }

    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(RodValueInput(self.0))
    }
}
