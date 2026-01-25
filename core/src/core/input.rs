use crate::core::value::RodValue;
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
    Undefined,
    Unknown,
}

#[cfg(not(feature = "non-thread-safe"))]
pub trait RodThreadBounds: Send + Sync {}
#[cfg(not(feature = "non-thread-safe"))]
impl<T: Send + Sync + ?Sized> RodThreadBounds for T {}

#[cfg(feature = "non-thread-safe")]
pub trait RodThreadBounds {}
#[cfg(feature = "non-thread-safe")]
impl<T: ?Sized> RodThreadBounds for T {}

/// The data access trait.
pub trait RodInput<'a>: std::fmt::Debug + RodThreadBounds {
    fn get_type(&self) -> DataType;

    fn as_str(&self) -> Option<Cow<'a, str>>;
    fn as_f64(&self) -> Option<f64>;
    fn as_i64(&self) -> Option<i64>;
    fn as_u64(&self) -> Option<u64>;
    fn as_bool(&self) -> Option<bool>;

    fn as_str_len(&self) -> Option<usize> {
        self.as_str().map(|s| s.len())
    }

    fn get_js_value(&self) -> Option<wasm_bindgen::JsValue> {
        None
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>>;

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>>;

    fn count(&self) -> Option<usize>;
    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>>;
    fn to_json(&self) -> serde_json::Value;
    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a>;
}

/// A Sized wrapper for dynamic trait objects (&dyn RodInput).
///
/// This is used as a bridge to allow dynamic trait objects to enter
/// the generic/monomorphized validation pipeline which requires Sized types.
#[derive(Debug)]
pub struct BoxedInput<'a, 'b>(pub &'a dyn RodInput<'b>);

// Note: RodThreadBounds is automatically implemented for BoxedInput
// via the blanket implementation in this file. No manual impl needed.

impl<'a, 'b> RodInput<'b> for BoxedInput<'a, 'b> {
    fn get_type(&self) -> DataType {
        self.0.get_type()
    }
    fn as_str(&self) -> Option<Cow<'b, str>> {
        self.0.as_str()
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
    fn as_str_len(&self) -> Option<usize> {
        self.0.as_str_len()
    }
    fn get_js_value(&self) -> Option<wasm_bindgen::JsValue> {
        self.0.get_js_value()
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'b>) -> Result<RodValue<'b>, ()>,
    ) -> Option<Result<RodValue<'b>, ()>> {
        self.0.with_key(key, f)
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'b>) -> Result<RodValue<'b>, ()>,
    ) -> Option<Result<RodValue<'b>, ()>> {
        self.0.with_index(index, f)
    }

    fn count(&self) -> Option<usize> {
        self.0.count()
    }
    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'b, str>> + '_>> {
        self.0.keys()
    }
    fn to_json(&self) -> serde_json::Value {
        self.0.to_json()
    }
    fn clone_box(&self) -> Box<dyn RodInput<'b> + 'b> {
        self.0.clone_box()
    }
}

impl<'a> Clone for Box<dyn RodInput<'a> + 'a> {
    fn clone(&self) -> Box<dyn RodInput<'a> + 'a> {
        self.clone_box()
    }
}
