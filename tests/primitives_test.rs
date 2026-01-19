use chrono::{TimeZone, Utc};
use rod::{RodValidator, coerce, date, string};
use serde_json::json;

#[test]
fn test_string_extended() {
    let schema = string()
        .length(5)
        .starts_with("h")
        .ends_with("o")
        .includes("ell");

    assert!(schema.validate(&json!("hello")).is_ok());
    assert!(schema.validate(&json!("hallo")).is_err());
    assert!(schema.validate(&json!("hellos")).is_err());

    let ip_schema = string().ip();
    assert!(ip_schema.validate(&json!("192.168.1.1")).is_ok());
    assert!(ip_schema.validate(&json!("999.999.999.999")).is_err());

    let date_string_schema = string().datetime();
    assert!(
        date_string_schema
            .validate(&json!("2023-01-01T00:00:00Z"))
            .is_ok()
    );
    assert!(date_string_schema.validate(&json!("not-a-date")).is_err());
}

#[test]
fn test_date() {
    let now = Utc::now().timestamp_millis();
    let past = Utc.timestamp_millis_opt(0).unwrap().timestamp_millis(); // 1970

    let schema = date().min(past).max(now);

    assert!(schema.validate(&json!("2000-01-01T00:00:00Z")).is_ok());
    assert!(schema.validate(&json!("not-a-date")).is_err());

    let future_schema = date().min(now);
    assert!(
        future_schema
            .validate(&json!("1980-01-01T00:00:00Z"))
            .is_err()
    );
}

#[test]
fn test_coerce() {
    let schema = coerce::string();
    let res = schema.validate(&json!(1234));
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), json!("1234"));

    let num_schema = coerce::number();
    let res_num = num_schema.validate(&json!("42"));
    assert!(res_num.is_ok());
    assert_eq!(res_num.unwrap(), json!(42));
}
