use rod::io::json::wrap;
use rod::types::node::IntoRodNode;
use rod::types::object::RodObject;
use rod::{
    RodValidator, RodValue, any, enum_type, intersection, lazy, literal, never, number, refine,
    rod_obj, string, transform, union,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_union_literal_enum() {
    // UPDATED: No more Box::new() for union options
    let role_schema = union(vec![literal("admin"), literal("user")]);
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
    // UPDATED: Manually constructing using RodNode for static dispatch
    let mut map_a = HashMap::new();
    map_a.insert("name".to_string(), string().into_node());
    let schema_a = RodObject::new(map_a);

    let mut map_b = HashMap::new();
    map_b.insert("age".to_string(), number().into_node());
    let schema_b = RodObject::new(map_b);

    let schema = intersection(schema_a, schema_b);

    let valid = json!({ "name": "Rod", "age": 1 });
    let result = schema.validate(&wrap(&valid));
    assert!(result.is_ok());

    let output = result.unwrap().to_json();
    assert_eq!(output.get("name").unwrap(), "Rod");
    assert_eq!(output.get("age").unwrap(), 1.0);
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
        |v| RodValue::Json(json!(v.as_str().unwrap().to_uppercase())),
    );
    let valid = json!("racecar");
    let input = wrap(&valid);
    let result = schema.validate(&input);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_json(), json!("RACECAR"));
}

#[test]
fn test_lazy() {
    let schema = lazy(|| string().into_node());
    assert!(schema.validate(&wrap(&json!("hello"))).is_ok());
    assert!(schema.validate(&wrap(&json!(123))).is_err());
}

#[test]
fn test_discriminated_union() {
    use rod::{discriminated_union, literal, number, rod_obj};

    let circle = rod_obj! { kind: literal("circle"), radius: number() };
    let square = rod_obj! { kind: literal("square"), side: number() };

    let shapes = discriminated_union("kind", vec![("circle", circle), ("square", square)]);

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
