// src/schema/parser.rs
use crate::core::validator::RodValidator;
use crate::types::{
    array, boolean, discriminated_union, enum_type, literal, map, number, object, record, set,
    string, tuple, union,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RodSpec {
    String {
        min: Option<usize>,
        max: Option<usize>,
        length: Option<usize>,
        email: Option<bool>,
        url: Option<bool>,
        uuid: Option<bool>,
        cuid: Option<bool>,
        datetime: Option<bool>,
        ip: Option<bool>,
        regex: Option<String>,
        starts_with: Option<String>,
        ends_with: Option<String>,
        includes: Option<String>,
        #[serde(default)]
        trim: bool,
        #[serde(default)]
        coerce: bool,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        int: Option<bool>,
    },
    Boolean,
    Array {
        items: Box<RodSpec>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Object {
        properties: HashMap<String, RodSpec>,
        strict: Option<bool>,
    },
    // --- Phase 3 Additions ---
    Union {
        options: Vec<RodSpec>,
    },
    Literal {
        value: Value,
    },
    Enum {
        values: Vec<String>,
    },
    Tuple {
        items: Vec<RodSpec>,
    },
    Record {
        key: Box<RodSpec>,
        value: Box<RodSpec>,
    },
    Set {
        value: Box<RodSpec>,
        min: Option<usize>,
    },
    Map {
        key: Box<RodSpec>,
        value: Box<RodSpec>,
    },
    DiscriminatedUnion {
        discriminator: String,
        options: Vec<RodSpec>,
    },
    Date {
        min: Option<i64>, // Pass timestamps in spec
        max: Option<i64>,
    },
}

impl RodSpec {
    pub fn build(&self) -> Box<dyn RodValidator> {
        match self {
            RodSpec::Date { min, max } => {
                let mut d = crate::types::date::date();
                if let Some(v) = min {
                    d = d.min(*v);
                }
                if let Some(v) = max {
                    d = d.max(*v);
                }
                Box::new(d)
            }
            RodSpec::String {
                min,
                max,
                length,
                email,
                url,
                uuid,
                cuid,
                datetime,
                ip,
                regex,
                starts_with,
                ends_with,
                includes,
                trim,
                coerce: _coerce,
            } => {
                let mut s = string::string();
                if let Some(v) = min {
                    s = s.min(*v);
                }
                if let Some(v) = max {
                    s = s.max(*v);
                }
                if let Some(v) = length {
                    s = s.length(*v);
                }

                if email.unwrap_or(false) {
                    s = s.email();
                }
                if url.unwrap_or(false) {
                    s = s.url();
                }
                if uuid.unwrap_or(false) {
                    s = s.uuid();
                }
                if cuid.unwrap_or(false) {
                    s = s.cuid();
                }
                if datetime.unwrap_or(false) {
                    s = s.datetime();
                }
                if ip.unwrap_or(false) {
                    s = s.ip();
                }

                if let Some(v) = regex {
                    s = s.regex(v);
                }
                if let Some(v) = starts_with {
                    s = s.starts_with(v);
                }
                if let Some(v) = ends_with {
                    s = s.ends_with(v);
                }
                if let Some(v) = includes {
                    s = s.includes(v);
                }

                if *trim {
                    s = s.trim();
                }

                // ... coercion wrapper logic ...
                Box::new(s)
            }
            RodSpec::Number { min, max, int } => {
                let mut n = number::number();
                if let Some(v) = min {
                    n = n.min(*v);
                }
                if let Some(v) = max {
                    n = n.max(*v);
                }
                if int.unwrap_or(false) {
                    n = n.int();
                }
                Box::new(n)
            }
            RodSpec::Boolean => Box::new(boolean::boolean()),
            RodSpec::Array { items, min, max } => {
                let mut a = array::array(items.build());
                if let Some(v) = min {
                    a = a.min(*v);
                }
                if let Some(v) = max {
                    a = a.max(*v);
                }
                Box::new(a)
            }
            RodSpec::Object { properties, strict } => {
                let mut map = HashMap::new();
                for (k, v) in properties {
                    map.insert(k.clone(), v.build());
                }
                let mut obj = object::object(map);
                if strict.unwrap_or(false) {
                    obj = obj.strict();
                } else {
                    obj = obj.strip();
                }
                Box::new(obj)
            }
            // --- Phase 3 Logic ---
            RodSpec::Union { options } => {
                let validators = options.iter().map(|o| o.build()).collect();
                Box::new(union::union(validators))
            }
            RodSpec::Literal { value } => Box::new(literal::literal(value.clone())),
            RodSpec::Enum { values } => {
                let str_refs: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
                Box::new(enum_type::enum_type(str_refs))
            }
            RodSpec::Tuple { items } => {
                let validators = items.iter().map(|i| i.build()).collect();
                Box::new(tuple::tuple(validators))
            }
            RodSpec::Record { key, value } => Box::new(record::record(key.build(), value.build())),
            RodSpec::Set { value, min } => {
                let mut s = set::set(value.build());
                if let Some(m) = min {
                    s = s.min(*m);
                }
                Box::new(s)
            }
            RodSpec::Map { key, value } => Box::new(map::map(key.build(), value.build())),
            RodSpec::DiscriminatedUnion {
                discriminator,
                options,
            } => {
                let mut map = HashMap::new();

                for opt in options {
                    // In a real implementation, we must peek into 'opt' (which must be an Object spec)
                    // find the property matching 'discriminator', ensure it is a Literal spec,
                    // extract the value, and use that as the map key.

                    if let RodSpec::Object { properties, .. } = opt {
                        if let Some(RodSpec::Literal { value }) = properties.get(discriminator) {
                            if let Value::String(s) = value {
                                map.insert(s.clone(), opt.build());
                            }
                        }
                    }
                }

                Box::new(discriminated_union::discriminated_union_map(
                    discriminator.clone(),
                    map,
                ))
            }
        }
    }
}

pub fn from_yaml(content: &str) -> Result<Box<dyn RodValidator>, serde_yaml::Error> {
    let spec: RodSpec = serde_yaml::from_str(content)?;
    Ok(spec.build())
}
