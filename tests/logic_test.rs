use rod::io::json::wrap;
use rod::types::object::object;
use rod::{
    RodValidator, any, enum_type, intersection, lazy, literal, never, number, refine, rod_obj,
    string, transform, union,
};
use serde_json::json;
use std::collections::HashMap; // for manual map construction if needed

#[test]
fn test_union_literal_enum() {
    let role_schema = union(vec![Box::new(literal("admin")), Box::new(literal("user"))]);
    assert!(role_schema.validate(&wrap(&json!("admin"))).is_ok());
    assert!(role_schema.validate(&wrap(&json!("guest"))).is_err());

    let status_schema = enum_type(vec!["active", "inactive"]);
    assert!(status_schema.validate(&wrap(&json!("active"))).is_ok());
    assert!(status_schema.validate(&wrap(&json!("deleted"))).is_err());

    let any_schema = any();
    assert!(any_schema.validate(&wrap(&json!("whatever"))).is_ok());

    let never_schema = never();
    assert!(never_schema.validate(&wrap(&json!("anything"))).is_err());
}

#[test]
fn test_intersection() {
    // Manually constructing for intersection test to ensure types match perfectly
    let mut map_a = HashMap::new();
    map_a.insert(
        "name".to_string(),
        Box::new(string()) as Box<dyn RodValidator>,
    );
    let schema_a = object(map_a);

    let mut map_b = HashMap::new();
    map_b.insert(
        "age".to_string(),
        Box::new(number()) as Box<dyn RodValidator>,
    );
    let schema_b = object(map_b);

    let schema = intersection(schema_a, schema_b);

    let valid = json!({ "name": "Rod", "age": 1 });
    let result = schema.validate(&wrap(&valid));
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.get("name").unwrap(), "Rod");
    assert_eq!(output.get("age").unwrap(), 1);
}

#[test]
fn test_refine_transform() {
    let schema = transform(
        refine(string(), |v| {
            let s = v.as_str().unwrap();
            let rev: String = s.chars().rev().collect();
            if s == rev {
                Ok(())
            } else {
                Err("Not a palindrome".into())
            }
        }),
        |v| json!(v.as_str().unwrap().to_uppercase()),
    );

    let res = schema.validate(&wrap(&json!("racecar")));
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), json!("RACECAR"));
}

#[test]
fn test_lazy() {
    // Simple lazy test
    let schema = lazy(|| Box::new(string()));
    assert!(schema.validate(&wrap(&json!("hello"))).is_ok());
    assert!(schema.validate(&wrap(&json!(123))).is_err());
}

#[test]
fn test_discriminated_union() {
    use rod::{discriminated_union, literal, number, rod_obj}; // Assuming discriminated_union is exported

    let circle = rod_obj! { kind: literal("circle"), radius: number() };
    let square = rod_obj! { kind: literal("square"), side: number() };

    let shapes = discriminated_union(
        "kind",
        vec![("circle", Box::new(circle)), ("square", Box::new(square))],
    );

    assert!(
        shapes
            .validate(&wrap(&json!({ "kind": "circle", "radius": 10 })))
            .is_ok()
    );
    assert!(
        shapes
            .validate(&wrap(&json!({ "kind": "triangle", "side": 10 })))
            .is_err()
    );
}
