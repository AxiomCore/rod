use rod::core::input::{DataType, RodInput};
use rod::core::value::RodValue;
use std::borrow::Cow;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub struct JsInput {
    pub value: JsValue,
}

impl<'a> RodInput<'a> for JsInput {
    fn as_str_len(&self) -> Option<usize> {
        // Reflect::get fails on primitives. We cast to JsString instead.
        if self.value.is_string() {
            Some(self.value.unchecked_ref::<js_sys::JsString>().length() as usize)
        } else {
            None
        }
    }

    fn get_js_value(&self) -> Option<JsValue> {
        Some(self.value.clone())
    }

    // Also update clone_box implementation
    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> DataType {
        if self.value.is_string() {
            DataType::String
        } else if self.value.as_f64().is_some() {
            DataType::Number
        } else if self.value.as_bool().is_some() {
            DataType::Boolean
        } else if self.value.is_null() {
            DataType::Null
        } else if self.value.is_undefined() {
            DataType::Undefined
        } else if js_sys::Array::is_array(&self.value) {
            DataType::Array
        } else if self.value.is_object() {
            DataType::Object
        } else {
            DataType::Unknown
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        self.value.as_string().map(Cow::Owned)
    }

    fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }

    fn as_i64(&self) -> Option<i64> {
        self.value.as_f64().map(|f| f as i64)
    }

    fn as_u64(&self) -> Option<u64> {
        self.value.as_f64().map(|f| f as u64)
    }

    fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        let key_js = JsValue::from_str(key);

        match js_sys::Reflect::get(&self.value, &key_js) {
            Ok(val) if !val.is_undefined() => {
                let input = JsInput { value: val };
                Some(f(&input))
            }
            _ => None,
        }
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        let val = js_sys::Reflect::get_u32(&self.value, index as u32).ok()?;

        if val.is_undefined() {
            return None;
        }

        let input = JsInput { value: val };
        Some(f(&input))
    }

    fn count(&self) -> Option<usize> {
        if js_sys::Array::is_array(&self.value) {
            Some(self.value.unchecked_ref::<js_sys::Array>().length() as usize)
        } else if self.value.is_object() {
            Some(js_sys::Object::keys(self.value.unchecked_ref()).length() as usize)
        } else {
            None
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>> {
        if self.value.is_object() && !self.value.is_null() {
            let keys_array = js_sys::Object::keys(self.value.unchecked_ref());
            let len = keys_array.length();
            let mut i = 0;

            let iter = std::iter::from_fn(move || {
                if i >= len {
                    return None;
                }
                let key_js = keys_array.get(i);
                i += 1;
                key_js.as_string().map(Cow::Owned)
            });

            Some(Box::new(iter))
        } else {
            None
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_wasm_bindgen::from_value(self.value.clone()).unwrap_or(serde_json::Value::Null)
    }
}
