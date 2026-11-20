//! Check-only validation (`check`) must agree exactly with `validate`:
//! same accept/reject and the same issues (code, path, message, order).
use rod_rs::io::json::wrap;
use rod_rs::RodSpec;
use serde_json::{json, Value};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn random_spec(r: &mut Lcg, depth: u32) -> RodSpec {
    let k = if depth > 3 { r.below(4) } else { r.below(9) };
    match k {
        0 => RodSpec::String {
            min: (r.below(2) == 0).then(|| r.below(4) as usize),
            max: (r.below(2) == 0).then(|| r.below(8) as usize),
            length: None,
            email: Some(r.below(6) == 0),
            url: None,
            uuid: None,
            cuid: None,
            datetime: None,
            ip: None,
            regex: (r.below(5) == 0).then(|| "^[a-z]+$".to_string()),
            starts_with: None,
            ends_with: None,
            includes: None,
            trim: false,
        },
        1 => RodSpec::Number {
            min: (r.below(2) == 0).then(|| r.below(5) as f64),
            max: (r.below(2) == 0).then(|| 3.0 + r.below(50) as f64),
            int: Some(r.below(2) == 0),
        },
        2 => RodSpec::Boolean,
        3 => RodSpec::Enum { values: vec!["a".into(), "b".into()] },
        4 | 5 => RodSpec::Array {
            items: Box::new(random_spec(r, depth + 1)),
            min: (r.below(3) == 0).then(|| r.below(3) as usize),
            max: (r.below(3) == 0).then(|| r.below(5) as usize),
        },
        6 => RodSpec::Optional(Box::new(random_spec(r, depth + 1))),
        7 => RodSpec::Nullable(Box::new(random_spec(r, depth + 1))),
        _ => {
            let mut properties = std::collections::HashMap::new();
            for i in 0..r.below(5) {
                properties.insert(format!("f{i}"), random_spec(r, depth + 1));
            }
            RodSpec::Object { properties, strict: Some(r.below(3) == 0) }
        }
    }
}

fn random_value(r: &mut Lcg, depth: u32) -> Value {
    let k = if depth > 4 { r.below(5) } else { r.below(8) };
    match k {
        0 => Value::Null,
        1 => json!(r.below(2) == 0),
        2 => json!(r.below(60) as f64 / if r.below(3) == 0 { 2.0 } else { 1.0 }),
        3 => json!(["", "a", "ab", "abcdefghij", "x@y.zz", "A1", "b"][r.below(7) as usize]),
        4 => json!(-(r.below(5) as i64)),
        5 | 6 => Value::Array((0..r.below(6)).map(|_| random_value(r, depth + 1)).collect()),
        _ => {
            let mut m = serde_json::Map::new();
            for i in 0..r.below(6) {
                m.insert(format!("f{}", i + r.below(2)), random_value(r, depth + 1));
            }
            Value::Object(m)
        }
    }
}

#[test]
fn check_agrees_with_validate_on_generated_models() {
    let mut r = Lcg(0xC14);
    let mut rejected = 0;
    let mut accepted = 0;
    for _ in 0..3000 {
        let spec = random_spec(&mut r, 0);
        let validator = spec.build();
        for _ in 0..10 {
            let value = if r.below(3) == 0 {
                random_value(&mut r, 0)
            } else {
                // Shaped like the spec so deep paths are exercised.
                shaped(&spec, &mut r, 0)
            };
            let full = validator.validate(&wrap(&value)).map(|_| ());
            let check = validator.check(&wrap(&value));
            match (&full, &check) {
                (Ok(()), Ok(())) => accepted += 1,
                (Err(a), Err(b)) => {
                    rejected += 1;
                    assert_eq!(a.issues, b.issues, "issues differ for {spec:?} on {value}");
                }
                _ => panic!("accept/reject differs for {spec:?} on {value}: {full:?} vs {check:?}"),
            }
        }
    }
    assert!(accepted > 1000 && rejected > 1000, "accepted {accepted} rejected {rejected}");
}

fn shaped(spec: &RodSpec, r: &mut Lcg, depth: u32) -> Value {
    if r.below(8) == 0 || depth > 6 {
        return random_value(r, depth);
    }
    match spec {
        RodSpec::Array { items, .. } => {
            Value::Array((0..r.below(6)).map(|_| shaped(items, r, depth + 1)).collect())
        }
        RodSpec::Object { properties, .. } => {
            let mut m = serde_json::Map::new();
            for (k, v) in properties {
                if r.below(6) != 0 {
                    m.insert(k.clone(), shaped(v, r, depth + 1));
                }
            }
            if r.below(5) == 0 {
                m.insert("extra".into(), json!(1));
            }
            Value::Object(m)
        }
        RodSpec::Optional(inner) | RodSpec::Nullable(inner) => shaped(inner, r, depth + 1),
        _ => random_value(r, depth + 4),
    }
}

#[test]
fn recursive_specs_agree() {
    let yaml = include_str!("../../core/tests/check_recursive.yaml");
    let v = rod_rs::schema::parser::from_yaml(yaml).unwrap();
    for value in [
        json!({"name": "ab", "children": [{"name": "cd", "children": []}]}),
        json!({"name": "a", "children": [{"name": "", "children": [{"name": 1}]}]}),
        json!({"children": "x"}),
    ] {
        let a = v.validate(&wrap(&value)).map(|_| ()).map_err(|e| e.issues);
        let b = v.check(&wrap(&value)).map_err(|e| e.issues);
        assert_eq!(a, b);
    }
}

/// 10k items with constrained fields (the runtime benchmark model).
/// Run with `cargo test --release --test check_test -- --nocapture timing`.
#[test]
fn timing_10k_items_check_vs_validate() {
    let yaml = include_str!("../../core/tests/check_itemlist.yaml");
    let v = rod_rs::schema::parser::from_yaml(yaml).unwrap();
    let items: Vec<Value> = (1..=10_000)
        .map(|i| json!({"id": i.to_string(), "name": format!("item {i}"), "price": i as f64 * 1.25, "qty": i % 100, "active": i % 2 == 0, "tags": [format!("t{}", i % 7), "bench"]}))
        .collect();
    let body = json!({"items": items, "total": 10_000});
    let t = std::time::Instant::now();
    let rounds = 20;
    for _ in 0..rounds {
        assert!(v.validate(&wrap(&body)).is_ok());
    }
    let full = t.elapsed() / rounds;
    let t = std::time::Instant::now();
    for _ in 0..rounds {
        assert!(v.check(&wrap(&body)).is_ok());
    }
    let check = t.elapsed() / rounds;
    eprintln!("10k items: validate {full:?}, check {check:?}");
}
