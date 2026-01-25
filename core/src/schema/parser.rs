use crate::core::validator::DynValidator;
use crate::types::array::RodArray;
use crate::types::boolean::RodBoolean;
use crate::types::date::RodDate;
use crate::types::discriminated_union::RodDiscriminatedUnion;
use crate::types::enum_type::RodEnum;
use crate::types::literal::RodLiteral;
use crate::types::map::RodMap;
use crate::types::node::RodNode;
use crate::types::number::RodNumber;
use crate::types::object::RodObject;
use crate::types::primitive::{RodAny, RodNever};
use crate::types::record::RodRecord;
use crate::types::set::RodSet;
use crate::types::string::RodString;
use crate::types::tuple::RodTuple;
use crate::types::union::RodUnion;

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

    /// Recursively builds the schema into an optimized RodNode tree.
    pub fn build_node(&self) -> RodNode {
        match self {
            RodSpec::Date { min, max } => {
                let mut d = RodDate::new();
                if let Some(v) = min {
                    d = d.min(*v);
                }
                if let Some(v) = max {
                    d = d.max(*v);
                }
                RodNode::Date(d)
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
                let mut s = RodString::new();
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
                RodNode::String(s)
            }
            RodSpec::Number { min, max, int } => {
                let mut n = RodNumber::new();
                if let Some(v) = min {
                    n = n.min(*v);
                }
                if let Some(v) = max {
                    n = n.max(*v);
                }
                if int.unwrap_or(false) {
                    n = n.int();
                }
                RodNode::Number(n)
            }
            RodSpec::Boolean => RodNode::Boolean(RodBoolean::default()),
            RodSpec::Any => RodNode::Any(RodAny::default()),
            RodSpec::Never => RodNode::Never(RodNever::default()),
            RodSpec::Array { items, min, max } => {
                let mut a = RodArray::new(items.build_node());
                if let Some(v) = min {
                    a = a.min(*v);
                }
                if let Some(v) = max {
                    a = a.max(*v);
                }
                RodNode::Array(a)
            }
            RodSpec::Object { properties, strict } => {
                let mut map = HashMap::new();
                for (k, v) in properties {
                    map.insert(k.clone(), v.build_node());
                }
                let mut obj = RodObject::new(map);
                if strict.unwrap_or(false) {
                    obj = obj.strict();
                } else {
                    obj = obj.strip();
                }
                RodNode::Object(obj)
            }
            RodSpec::Union { options } => RodNode::Union(RodUnion::new(
                options.iter().map(|o| o.build_node()).collect(),
            )),
            RodSpec::Literal { value } => RodNode::Literal(RodLiteral::new(value.clone())),
            RodSpec::Enum { values } => RodNode::Enum(RodEnum::new(values.clone())),
            RodSpec::Tuple { items } => RodNode::Tuple(RodTuple::new(
                items.iter().map(|i| i.build_node()).collect(),
            )),
            RodSpec::Record { key, value } => {
                RodNode::Record(RodRecord::new(key.build_node(), value.build_node()))
            }
            RodSpec::Set { value, min } => {
                let mut s = RodSet::new(value.build_node());
                if let Some(v) = min {
                    s = s.min(*v);
                }
                RodNode::Set(s)
            }
            RodSpec::Map { key, value } => {
                RodNode::Map(RodMap::new(key.build_node(), value.build_node()))
            }
            RodSpec::DiscriminatedUnion {
                discriminator,
                options,
            } => {
                let mut map = HashMap::new();
                for opt in options {
                    if let Some(val) = opt.find_discriminator_value(discriminator) {
                        map.insert(val, opt.build_node());
                    }
                }
                RodNode::DiscriminatedUnion(RodDiscriminatedUnion::new(discriminator.clone(), map))
            }
        }
    }

    /// Legacy support returning a boxed object-safe trait object.
    /// Changed from Box<dyn RodValidator> to Box<dyn DynValidator>.
    pub fn build(&self) -> Box<dyn DynValidator> {
        Box::new(self.build_node())
    }
}

/// Parse a Rod schema from YAML. Returns an object-safe trait object.
pub fn from_yaml(content: &str) -> Result<Box<dyn DynValidator>, serde_yaml::Error> {
    let spec: RodSpec = serde_yaml::from_str(content)?;
    Ok(spec.build())
}
