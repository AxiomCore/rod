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

// FIX: Removed Send + Sync bounds to allow WASM JsValue implementation
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

impl<'a> Clone for Box<dyn RodInput<'a> + 'a> {
    fn clone(&self) -> Box<dyn RodInput<'a> + 'a> {
        self.clone_box()
    }
}
