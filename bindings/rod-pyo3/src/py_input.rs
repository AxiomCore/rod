use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use pythonize::depythonize;
use rod::core::input::{DataType, RodInput};
use rod::core::value::RodValue;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct PyInput<'a>(pub Bound<'a, PyAny>);

impl<'a> RodInput<'a> for PyInput<'a> {
    fn get_type(&self) -> DataType {
        if self.0.is_none() {
            DataType::Null
        } else if self.0.is_instance_of::<PyString>() {
            DataType::String
        } else if self.0.is_instance_of::<PyBool>() {
            DataType::Boolean
        } else if self.0.is_instance_of::<PyFloat>() || self.0.is_instance_of::<PyInt>() {
            DataType::Number
        } else if self.0.is_instance_of::<PyList>() || self.0.is_instance_of::<PyTuple>() {
            DataType::Array
        } else if self.0.is_instance_of::<PyDict>() {
            DataType::Object
        } else {
            // Fallback for custom objects? Treat as unknown/object for now
            DataType::Unknown
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        let s = self.0.downcast::<PyString>().ok()?;
        let slice = s.to_str().ok()?;
        // SAFETY: Python strings are immutable and pinned in memory.
        // The Bound<'a> wrapper ensures the object remains alive for lifetime 'a.
        // We extend the lifetime of the slice from the local borrow to 'a to satisfy the trait.
        // This avoids allocation (Zero-Copy).
        let slice_a: &'a str = unsafe { std::mem::transmute(slice) };
        Some(Cow::Borrowed(slice_a))
    }

    fn as_f64(&self) -> Option<f64> {
        self.0.extract::<f64>().ok()
    }

    fn as_i64(&self) -> Option<i64> {
        self.0.extract::<i64>().ok()
    }

    fn as_u64(&self) -> Option<u64> {
        self.0.extract::<u64>().ok()
    }

    fn as_bool(&self) -> Option<bool> {
        self.0.extract::<bool>().ok()
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        if let Ok(dict) = self.0.downcast::<PyDict>() {
            match dict.get_item(key) {
                Ok(Some(item)) => {
                    // item is Bound<'a, PyAny>
                    let input = PyInput(item);
                    Some(f(&input))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        if let Ok(list) = self.0.downcast::<PyList>() {
            match list.get_item(index) {
                Ok(item) => {
                    let input = PyInput(item);
                    Some(f(&input))
                }
                _ => None,
            }
        } else if let Ok(tuple) = self.0.downcast::<PyTuple>() {
            match tuple.get_item(index) {
                Ok(item) => {
                    let input = PyInput(item);
                    Some(f(&input))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn count(&self) -> Option<usize> {
        if let Ok(list) = self.0.downcast::<PyList>() {
            Some(list.len())
        } else if let Ok(tuple) = self.0.downcast::<PyTuple>() {
            Some(tuple.len())
        } else if let Ok(dict) = self.0.downcast::<PyDict>() {
            Some(dict.len())
        } else {
            None
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>> {
        if let Ok(dict) = self.0.downcast::<PyDict>() {
            let keys_list = dict.keys();
            let mut keys_vec = Vec::with_capacity(keys_list.len());
            for item in keys_list.iter() {
                if let Ok(s) = item.downcast::<PyString>() {
                    if let Ok(utf8) = s.to_str() {
                        // SAFETY: Keys are immutable strings kept alive by the dict,
                        // which is kept alive by Bound<'a>. Zero-Copy keys.
                        let utf8_a: &'a str = unsafe { std::mem::transmute(utf8) };
                        keys_vec.push(Cow::Borrowed(utf8_a));
                    }
                }
            }
            Some(Box::new(keys_vec.into_iter()))
        } else {
            None
        }
    }

    fn to_json(&self) -> serde_json::Value {
        // Fallback to serialization if the validator needs to materialize the value (e.g. for return)
        depythonize(&self.0).unwrap_or(serde_json::Value::Null)
    }

    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(PyInput(self.0.clone()))
    }
}
