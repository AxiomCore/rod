#![allow(dead_code)]

#[cfg(feature = "macros")]
use rod_rs::{Rod, RodSchema};
use serde_json::json;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

// --- 1. Basic Struct ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct SimpleUser {
    #[rod(min = 3)]
    username: String,
    age: u8,
}

// --- 2. Smart Pointers & References ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct SmartData<'a> {
    boxed: Box<String>,
    rc: Rc<i32>,
    arc: Arc<bool>,
    cow: Cow<'a, str>,
    reference: &'a str,
}

// --- 3. Complex Containers ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct Collections {
    // HashSet -> rod::set
    unique_tags: HashSet<String>,

    // HashMap -> rod::record
    metadata: HashMap<String, i32>,

    // Nested Options: Option<Vec<Option<i32>>>
    matrix: Option<Vec<Option<i32>>>,
}

// --- 4. Tuples & Arrays ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct CoordinateSystem {
    // Tuple: (x, y, z) -> rod::tuple([number, number, number])
    point: (f64, f64, f64),

    // Fixed Array -> rod::array(number)
    matrix_row: [u8; 4],
}

// --- 5. Generics ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct Wrapper<T> {
    data: T,
    meta: String,
}

// --- 6. Nested Schemas ---
#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct NestedRoot {
    user: SimpleUser,
    wrapper: Wrapper<i32>,
}

#[test]
#[cfg(feature = "macros")]
fn test_macro_edge_cases() {
    // 1. Test Smart Pointers
    let smart_schema = SmartData::schema();
    let smart_data = json!({
        "boxed": "hello",
        "rc": 42,
        "arc": true,
        "cow": "cow_string",
        "reference": "ref_string"
    });
    assert!(
        smart_schema
            .validate(&rod_rs::io::json::wrap(&smart_data))
            .is_ok()
    );

    // 2. Test Collections
    let col_schema = Collections::schema();
    let col_data = json!({
        "unique_tags": ["a", "b", "c"],
        "metadata": { "key": 100 },
        "matrix": [ 1, null, 100 ]
    });

    assert!(
        col_schema
            .validate(&rod_rs::io::json::wrap(&col_data))
            .is_ok()
    );

    // Test Set Uniqueness
    let invalid_set = json!({
        "unique_tags": ["a", "a"], // Duplicate!
        "metadata": {},
        "matrix": null
    });
    assert!(
        col_schema
            .validate(&rod_rs::io::json::wrap(&invalid_set))
            .is_err()
    );

    // 3. Test Tuples & Arrays
    let coord_schema = CoordinateSystem::schema();
    let coord_data = json!({
        "point": [1.0, 2.0, 3.0],
        "matrix_row": [1, 2, 3, 4]
    });
    assert!(
        coord_schema
            .validate(&rod_rs::io::json::wrap(&coord_data))
            .is_ok()
    );

    // Invalid Tuple (wrong types)
    let invalid_tuple = json!({
        "point": ["1", 2.0, 3.0], // String instead of number
        "matrix_row": []
    });
    assert!(
        coord_schema
            .validate(&rod_rs::io::json::wrap(&invalid_tuple))
            .is_err()
    );

    // 4. Test Generics & Nesting
    let root_schema = NestedRoot::schema();
    let root_data = json!({
        "user": {
            "username": "Rod",
            "age": 10
        },
        "wrapper": {
            "data": 999, // T is i32
            "meta": "info"
        }
    });
    assert!(
        root_schema
            .validate(&rod_rs::io::json::wrap(&root_data))
            .is_ok()
    );
}
