use rod::io::json::wrap;
use rod::{
    NullableExtension, OptionalExtension, RodValidator, boolean, number, record, rod_obj, string,
};
use serde_json::json;

#[test]
fn test_macro_object() {
    // Note: 'array' needs to be imported if used inside macro,
    // but the macro expands to `rod::types::object::object`, so imports matter.
    // Let's import array here.
    use rod::array;

    let schema = rod_obj! {
        name: string().min(2),
        age: number().int().min(0.0),
        isActive: boolean(),
        tags: array(string()).min(1)
    };

    let valid = json!({
        "name": "Rod",
        "age": 42,
        "isActive": true,
        "tags": ["rust", "zod"]
    });

    assert!(schema.validate(&wrap(&valid)).is_ok());

    let invalid = json!({
        "name": "R",
        "age": 42.5,
        "isActive": "yes",
        "tags": []
    });

    let err = schema.validate(&wrap(&invalid)).unwrap_err();
    assert_eq!(err.issues.len(), 4);
}

#[test]
fn test_record() {
    let schema = record(string().min(2), number());
    assert!(
        schema
            .validate(&wrap(&json!({ "score": 10, "hp": 100 })))
            .is_ok()
    );
    assert!(schema.validate(&wrap(&json!({ "a": 1 }))).is_err()); // Key too short
}

#[test]
fn test_deep_partial() {
    let schema = rod_obj! {
        name: string(),
        nested: rod_obj! {
            age: number()
        }
    };

    let partial = schema.deep_partial();
    assert!(partial.validate(&wrap(&json!({}))).is_ok());
    assert!(partial.validate(&wrap(&json!({ "nested": {} }))).is_ok());
}

#[test]
fn test_keyof() {
    let schema = rod_obj! {
        name: string(),
        age: number()
    };
    let keys = schema.keyof();
    assert!(keys.validate(&wrap(&json!("name"))).is_ok());
    assert!(keys.validate(&wrap(&json!("email"))).is_err());
}

#[test]
fn test_optional_nullable() {
    let schema = rod_obj! {
        req: string(),
        opt: string().optional(),
        null: string().nullable()
    };

    assert!(
        schema
            .validate(&wrap(&json!({ "req": "a", "opt": "b", "null": "c" })))
            .is_ok()
    );
    assert!(
        schema
            .validate(&wrap(&json!({ "req": "a", "null": null })))
            .is_ok()
    ); // opt missing is ok
    assert!(
        schema
            .validate(&wrap(&json!({ "opt": "b", "null": "c" })))
            .is_err()
    ); // req missing
}
