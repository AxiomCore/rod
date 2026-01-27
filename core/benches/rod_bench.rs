use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rod_rs::core::validator::RodValidator;
use rod_rs::io::json::wrap;
use rod_rs::types::{array::array, number::number, object::object, string::string};
use serde_json::json;
use std::collections::HashMap;

fn bench_simple_object(c: &mut Criterion) {
    let mut shape = HashMap::new();
    shape.insert(
        "name".to_string(),
        Box::new(string()) as Box<dyn RodValidator>,
    );
    shape.insert(
        "age".to_string(),
        Box::new(number()) as Box<dyn RodValidator>,
    );
    let schema = object(shape);

    let data = json!({ "name": "John Doe", "age": 30 });
    let input = wrap(&data);

    c.bench_function("simple_object", |b| {
        b.iter(|| {
            let _ = schema.validate(black_box(&input));
        })
    });
}

fn bench_heavy_logic(c: &mut Criterion) {
    let mut shape = HashMap::new();
    shape.insert(
        "id".to_string(),
        Box::new(string().uuid()) as Box<dyn RodValidator>,
    );
    shape.insert(
        "email".to_string(),
        Box::new(string().email()) as Box<dyn RodValidator>,
    );
    shape.insert(
        "tags".to_string(),
        Box::new(array(string().min(3)).min(10)) as Box<dyn RodValidator>,
    );
    let schema = object(shape);

    let data = json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "email": "performance@rod.rs",
        "tags": vec!["rust-lang"; 12]
    });
    let input = wrap(&data);

    c.bench_function("heavy_logic_uuid_email", |b| {
        b.iter(|| {
            let _ = schema.validate(black_box(&input));
        })
    });
}

fn bench_massive_array(c: &mut Criterion) {
    let schema = array(number());
    let data = json!(vec![42.0; 100000]);
    let input = wrap(&data);

    c.bench_function("massive_array_100000_nums", |b| {
        b.iter(|| {
            let _ = schema.validate(black_box(&input));
        })
    });
}

fn bench_mega_object(c: &mut Criterion) {
    let mut shape = HashMap::new();
    let mut data_map = serde_json::Map::new();

    for i in 0..50 {
        let key = format!("field_{}", i);
        shape.insert(key.clone(), Box::new(string()) as Box<dyn RodValidator>);
        data_map.insert(key, json!("value"));
    }

    let schema = object(shape);
    let data = serde_json::Value::Object(data_map);
    let input = wrap(&data);

    c.bench_function("mega_object_50_fields", |b| {
        b.iter(|| {
            let _ = schema.validate(black_box(&input));
        })
    });
}

criterion_group!(
    benches,
    bench_simple_object,
    bench_heavy_logic,
    bench_massive_array,
    bench_mega_object
);
criterion_main!(benches);
