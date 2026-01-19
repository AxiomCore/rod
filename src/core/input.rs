// src/core/input.rs
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

/// A zero-copy view into any data source.
/// Lifetimes: 'a is the lifetime of the data source.
pub trait RodInput: Debug {
    fn get_type(&self) -> DataType;

    // Primitives (Zero-Copy References where possible)
    fn as_str(&self) -> Option<&str>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;
    fn as_i64(&self) -> Option<i64>; // Useful for int checks

    // Structural Access
    // We return a Box<dyn RodInput> that borrows from the parent source
    fn get_key(&self, key: &str) -> Option<Box<dyn RodInput + '_>>;
    fn get_index(&self, index: usize) -> Option<Box<dyn RodInput + '_>>;

    // Collections
    fn count(&self) -> Option<usize>;

    // Iterator for object keys (needed for strict/strip logic)
    fn keys(&self) -> Option<Box<dyn Iterator<Item = String> + '_>>;

    // For debugging/error reporting, we need a way to clone the value out
    fn to_json(&self) -> serde_json::Value;
}
