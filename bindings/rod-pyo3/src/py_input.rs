use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use pythonize::depythonize;
use rod::core::input::{DataType, RodInput};
use rod::core::value::RodValue;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct PyInput<'a>(pub Bound<'a, PyAny>);

/// Zero-allocation iterator for Python Dictionary keys.
/// We use Box<dyn Iterator> to abstract away the concrete PyO3 iterator type,
/// which ensures compatibility across PyO3 versions and ABI configurations.
pub struct PyKeyIterator<'a> {
    iter: Box<dyn Iterator<Item = (Bound<'a, PyAny>, Bound<'a, PyAny>)> + 'a>,
}

impl<'a> Iterator for PyKeyIterator<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // PyDictIterator yields (key, value) pairs.
            // We only need the key.
            let (key, _) = self.iter.next()?;

            // We attempt to cast to string. If it's not a string, we skip it
            // (Standard JSON/Object validation usually implies string keys).
            if let Ok(py_str) = key.downcast::<PyString>() {
                if let Ok(slice) = py_str.to_str() {
                    // SAFETY: The Python string object is kept alive by the dictionary,
                    // which is kept alive by Bound<'a>. The string data is pinned in memory.
                    // We extend the lifetime of the slice to 'a to satisfy the trait interface.
                    let slice_a: &'a str = unsafe { std::mem::transmute(slice) };
                    return Some(Cow::Borrowed(slice_a));
                }
            }
        }
    }
}

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
            // Fallback for custom objects
            DataType::Unknown
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        let s = self.0.downcast::<PyString>().ok()?;
        let slice = s.to_str().ok()?;
        // SAFETY: Python strings are immutable and pinned.
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
            // get_item returns Option<Bound>, avoiding Exception overhead.
            match dict.get_item(key) {
                Ok(Some(item)) => {
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
            // OPTIMIZATION: We use dict.iter() which is a native C-iterator.
            // We box it to hide the concrete type (PyDictIterator vs BoundDictIterator).
            // This incurs 1 allocation (the Box) instead of N allocations (keys list).
            let iter = Box::new(dict.iter());
            Some(Box::new(PyKeyIterator { iter }))
        } else {
            None
        }
    }

    fn to_json(&self) -> serde_json::Value {
        depythonize(&self.0).unwrap_or(serde_json::Value::Null)
    }

    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(PyInput(self.0.clone()))
    }
}
