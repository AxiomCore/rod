use rod::from_yaml;
use rod::io::json::wrap;
// IMPORTANT: RodValidator must be in scope to call .validate() on the boxed schema
use rod::RodValidator;
use serde_json::json;

#[test]
fn test_dynamic_complex_schema() {
    let yaml = r#"
type: object
strict: true
properties:
  user:
    type: object
    properties:
      id: 
        type: number
        int: true
      email:
        type: string
        email: true
  roles:
    type: array
    items:
      type: string
"#;
    // schema is Box<dyn DynValidator>
    let schema = from_yaml(yaml).expect("Spec parse failed");

    let valid = json!({
        "user": { "id": 1, "email": "test@rod.rs" },
        "roles": ["admin"]
    });

    // This calls the monomorphized validate path using JsonInput
    assert!(schema.validate(&wrap(&valid)).is_ok());

    let invalid = json!({
        "user": { "id": 1, "email": "test@rod.rs" },
        "roles": [],
        "extra": "fail" // Strict mode
    });

    let err = schema.validate(&wrap(&invalid)).unwrap_err();
    assert!(
        err.issues
            .iter()
            .any(|i| matches!(i.details, rod::error::RodIssueCode::UnrecognizedKeys { .. }))
    );
}

#[test]
fn test_dynamic_union_enum() {
    let yaml = r#"
type: object
properties:
  status:
    type: enum
    values: ["pending", "done"]
  result:
    type: union
    options:
      - type: string
      - type: number
"#;
    let schema = from_yaml(yaml).expect("Spec parse failed");

    assert!(
        schema
            .validate(&wrap(
                &json!({ "status": "pending", "result": "waiting..." })
            ))
            .is_ok()
    );
    assert!(
        schema
            .validate(&wrap(&json!({ "status": "done", "result": 42 })))
            .is_ok()
    );
    assert!(
        schema
            .validate(&wrap(&json!({ "status": "unknown", "result": 42 })))
            .is_err()
    );
}
