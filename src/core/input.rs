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

/// 'a is the lifetime of the underlying data source.
pub trait RodInput<'a>: Debug + Send + Sync {
    fn get_type(&self) -> DataType;

    // Zero-Copy accessors: Return data living as long as 'a
    fn as_str(&self) -> Option<Cow<'a, str>>;

    // Primitives
    fn as_f64(&self) -> Option<f64>;
    fn as_i64(&self) -> Option<i64>;
    fn as_bool(&self) -> Option<bool>;

    // Traversal
    // Returns a Box containing an implementation of RodInput tied to the same lifetime 'a
    fn get_key(&self, key: &str) -> Option<Box<dyn RodInput<'a> + '_>>;
    fn get_index(&self, index: usize) -> Option<Box<dyn RodInput<'a> + '_>>;

    fn count(&self) -> Option<usize>;
    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>>;

    // For error reporting
    fn to_json(&self) -> serde_json::Value;
}
