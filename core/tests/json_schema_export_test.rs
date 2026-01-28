#![allow(dead_code)]

#[cfg(feature = "macros")]
use rod_rs::Rod;
use serde_json::json;

#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct NestedConfig {
    #[rod(min = 1)]
    id: u32,
    active: bool,
}

#[cfg(feature = "macros")]
#[derive(Rod, Debug)]
struct UserProfile {
    #[rod(min = 3, max = 20)]
    username: String,

    #[rod(email)]
    email: String,

    #[rod(cuid)]
    cuid: String,

    #[rod(min = 18)]
    age: u8,

    nickname: Option<String>,

    tags: Vec<String>,

    config: NestedConfig,
}

#[test]
#[cfg(feature = "macros")]
fn test_export_to_json_schema_draft_2020_12() {
    let schema = UserProfile::json_schema();

    let expected = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["age", "config", "cuid", "email", "tags", "username"],
        "properties": {
            "username": {
                "type": "string",
                "minLength": 3,
                "maxLength": 20
            },
            "email": {
                "type": "string",
                "format": "email"
            },
            "cuid": {
                "type": "string",
                "pattern": "^[cC][^\\s-]{8,}$"
            },
            "age": {
                "type": "integer",
                "minimum": 18.0
            },
            "nickname": {
                "type": ["string", "null"]
            },
            "tags": {
                "type": "array",
                "items": {
                    "type": "string"
                }
            },
            "config": {
                "type": "object",
                "required": ["active", "id"], // Sorted: active, then id
                "properties": {
                    "id": {
                        "type": "integer",
                        "minimum": 1.0
                    },
                    "active": {
                        "type": "boolean"
                    }
                }
            }
        }
    });

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.iter().any(|v| v == "username"));
    assert!(
        !required.iter().any(|v| v == "nickname"),
        "Optional field should not be required"
    );

    assert_eq!(schema["properties"]["email"]["format"], "email");

    assert_eq!(schema["properties"]["cuid"]["pattern"], "^[cC][^\\s-]{8,}$");

    let nickname_type = &schema["properties"]["nickname"]["type"];
    assert!(nickname_type.is_array());
    assert!(nickname_type.as_array().unwrap().contains(&json!("null")));

    assert_eq!(schema, expected);
}

#[test]
#[cfg(feature = "macros")]
fn test_tuple_export_prefix_items() {
    #[derive(Rod)]
    struct Loc(f64, f64);

    use rod_rs::RodSpec;
    let tuple_spec = RodSpec::Tuple {
        items: vec![
            RodSpec::Number {
                min: None,
                max: None,
                int: Some(false),
            },
            RodSpec::String {
                min: None,
                max: None,
                length: None,
                email: None,
                url: None,
                uuid: None,
                cuid: None,
                datetime: None,
                ip: None,
                regex: None,
                starts_with: None,
                ends_with: None,
                includes: None,
                trim: false,
            },
        ],
    };

    let schema = tuple_spec.to_json_schema();

    assert!(schema.get("prefixItems").is_some());
    assert_eq!(schema["minItems"], 2);
    assert_eq!(schema["items"], false);
}

#[test]
fn test_complex_macro_builder_to_json_schema() {
    use rod_rs::{RodValidator, array, boolean, number, rod_obj, string};

    let schema = rod_obj! {
        id: string().uuid(),
        username: string().min(3).max(20),
        age: number().int().min(18.0),
        tags: array(string().min(2)).min(1),
        is_verified: boolean()
    };

    // Generate the JSON Schema
    let json = schema.json_schema();

    let expected_json = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["age", "id", "is_verified", "tags", "username"],
        "properties": {
            "id": {
                "type": "string",
                "format": "uuid"
            },
            "username": {
                "type": "string",
                "minLength": 3,
                "maxLength": 20
            },
            "age": {
                "type": "integer",
                "minimum": 18.0
            },
            "is_verified": {
                "type": "boolean"
            },
            "tags": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "string",
                    "minLength": 2
                }
            }
        }
    });

    assert_eq!(json, expected_json);
}
