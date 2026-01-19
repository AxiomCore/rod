use rod::{RodValidator, map, number, set, string, tuple};
use serde_json::json;

#[test]
fn test_tuple() {
    let point_schema = tuple(vec![Box::new(string()), Box::new(number())]);
    assert!(point_schema.validate(&json!(["lat", 45.0])).is_ok());
    assert!(point_schema.validate(&json!(["lat", "long"])).is_err());
}

#[test]
fn test_map_and_set() {
    let set_schema = set(number());
    assert!(set_schema.validate(&json!([1, 2, 3])).is_ok());
    assert!(set_schema.validate(&json!([1, 1, 2])).is_err());

    let map_schema = map(string(), number());
    let valid_map = json!([["key1", 10], ["key2", 20]]);
    assert!(map_schema.validate(&valid_map).is_ok());

    let invalid_key = json!([[123, 10]]);
    assert!(map_schema.validate(&invalid_key).is_err());
}
