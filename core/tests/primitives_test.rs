use chrono::{TimeZone, Utc};
use rod_rs::io::json::wrap;
use rod_rs::{RodValidator, coerce, date, string};
use serde_json::json;

#[test]
fn test_string_extended() {
    let schema = string()
        .length(5)
        .starts_with("h")
        .ends_with("o")
        .includes("ell");

    assert!(schema.validate(&wrap(&json!("hello"))).is_ok());
    assert!(schema.validate(&wrap(&json!("hallo"))).is_err());
    assert!(schema.validate(&wrap(&json!("hellos"))).is_err());

    let ip_schema = string().ip();
    assert!(ip_schema.validate(&wrap(&json!("192.168.1.1"))).is_ok());
    assert!(
        ip_schema
            .validate(&wrap(&json!("999.999.999.999")))
            .is_err()
    );

    let date_string_schema = string().datetime();
    assert!(
        date_string_schema
            .validate(&wrap(&json!("2023-01-01T00:00:00Z")))
            .is_ok()
    );
    assert!(
        date_string_schema
            .validate(&wrap(&json!("not-a-date")))
            .is_err()
    );
}

#[test]
fn test_date() {
    let now = Utc::now().timestamp_millis();
    let past = Utc.timestamp_millis_opt(0).unwrap().timestamp_millis(); // 1970

    let schema = date().min(past).max(now);

    assert!(
        schema
            .validate(&wrap(&json!("2000-01-01T00:00:00Z")))
            .is_ok()
    );
    assert!(schema.validate(&wrap(&json!("not-a-date"))).is_err());

    let future_schema = date().min(now);
    assert!(
        future_schema
            .validate(&wrap(&json!("1980-01-01T00:00:00Z")))
            .is_err()
    );
}

#[test]
fn test_coerce() {
    let schema = coerce::string();
    let valid = json!(1234);
    let input = wrap(&valid);
    let result = schema.validate(&input);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_json(), json!("1234"));

    let num_schema = coerce::number();
    let valid_num = json!(42);
    let input_num = wrap(&valid_num);
    let result_num = num_schema.validate(&input_num);
    assert!(result_num.is_ok());
    assert_eq!(result_num.unwrap().to_json(), json!(42.0));
}
