#![allow(dead_code)]

use rod_rs::RodValidator;
#[cfg(feature = "macros")]
use rod_rs::{Rod, RodSchema};
use serde_json::json;

#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct Address {
    #[rod(min = 2)]
    city: String,

    #[rod(min = 1000, max = 9999)]
    zip: u32,
}

#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct User {
    #[rod(min = 3)]
    username: String,

    #[rod(email)]
    email: String,

    #[rod(min = 18)]
    age: u32,

    is_active: bool,

    #[rod(max = 50)]
    nickname: Option<String>,

    tags: Vec<String>,

    address: Address,
}

#[test]
#[cfg(feature = "macros")]
fn test_derive_macro() {
    let schema = User::schema();

    // 1. Valid Data
    let valid = json!({
        "username": "Rod",
        "email": "test@rod.rs",
        "age": 25,
        "is_active": true,
        "nickname": "Roddy",
        "tags": ["rust", "macro"],
        "address": {
            "city": "NY",
            "zip": 1001
        }
    });

    assert!(schema.validate(&rod_rs::io::json::wrap(&valid)).is_ok());

    // 2. Invalid Data (Check recursive validation)
    let invalid = json!({
        "username": "Rod",
        "email": "test@rod.rs",
        "age": 25,
        "is_active": true,
        "tags": [],
        "address": {
            "city": "NY",
            "zip": 999 // Too small (min 1000)
        }
    });

    let err = schema
        .validate(&rod_rs::io::json::wrap(&invalid))
        .unwrap_err();
    // Verify the error is coming from the nested struct
    let issue = &err.issues[0];
    assert!(issue.path.contains(&"address".to_string()));
    assert!(issue.path.contains(&"zip".to_string()));
}
