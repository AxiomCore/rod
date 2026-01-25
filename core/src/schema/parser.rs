use crate::core::validator::RodValidator;
// Explicitly import factory functions to avoid module name clashes
use crate::types::array::array;
use crate::types::boolean::boolean;
use crate::types::date::date;
use crate::types::discriminated_union::discriminated_union_map;
use crate::types::literal::literal;
use crate::types::map::map;
use crate::types::number::number;
use crate::types::object::object;
use crate::types::primitive::{any, never};
use crate::types::record::record;
use crate::types::set::set;
use crate::types::string::string;
use crate::types::tuple::tuple;
use crate::types::union::union;

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
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        int: Option<bool>,
    },
    Boolean,
    Any,
    Never,
    Array {
        items: Box<RodSpec>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Object {
        properties: HashMap<String, RodSpec>,
        strict: Option<bool>,
    },
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
        min: Option<i64>,
        max: Option<i64>,
    },
}

impl RodSpec {
    fn find_discriminator_value(&self, key: &str) -> Option<String> {
        match self {
            RodSpec::Object { properties, .. } => {
                if let Some(spec) = properties.get(key) {
                    if let RodSpec::Literal { value } = spec {
                        return value.as_str().map(|s| s.to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn build(&self) -> Box<dyn RodValidator> {
        match self {
            RodSpec::Date { min, max } => {
                let mut d = date();
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
            } => {
                let mut s = string();
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
                Box::new(s)
            }
            RodSpec::Number { min, max, int } => {
                let mut n = number();
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
            RodSpec::Boolean => Box::new(boolean()),
            RodSpec::Any => Box::new(any()),
            RodSpec::Never => Box::new(never()),
            RodSpec::Array { items, min, max } => {
                let mut a = array(items.build());
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
                let mut obj = object(map);
                if strict.unwrap_or(false) {
                    obj = obj.strict();
                } else {
                    obj = obj.strip();
                }
                Box::new(obj)
            }
            RodSpec::Union { options } => {
                Box::new(union(options.iter().map(|o| o.build()).collect()))
            }
            RodSpec::Literal { value } => Box::new(literal(value.clone())),
            RodSpec::Enum { values } => {
                Box::new(crate::types::enum_type::RodEnum::new(values.clone()))
            }
            RodSpec::Tuple { items } => Box::new(tuple(items.iter().map(|i| i.build()).collect())),
            RodSpec::Record { key, value } => Box::new(record(key.build(), value.build())),
            RodSpec::Set { value, min } => {
                let mut s = set(value.build());
                if let Some(v) = min {
                    s = s.min(*v);
                }
                Box::new(s)
            }
            RodSpec::Map { key, value } => Box::new(map(key.build(), value.build())),
            RodSpec::DiscriminatedUnion {
                discriminator,
                options,
            } => {
                let mut map = HashMap::new();
                for opt in options {
                    if let Some(val) = opt.find_discriminator_value(discriminator) {
                        map.insert(val, opt.build());
                    } else {
                        // For simplicity in parser, assume spec is correct or ignore
                    }
                }
                Box::new(discriminated_union_map(discriminator.clone(), map))
            }
        }
    }
}

pub fn from_yaml(content: &str) -> Result<Box<dyn RodValidator>, serde_yaml::Error> {
    let spec: RodSpec = serde_yaml::from_str(content)?;
    Ok(spec.build())
}
