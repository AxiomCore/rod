use crate::core::validator::RodValidator;
use crate::schema::spec::RodSpec;
use crate::{any, boolean, number, string};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

pub trait RodSchema {
    fn schema() -> Box<dyn RodValidator>;
    fn spec() -> RodSpec;

    /// Generates a JSON Schema for the type
    fn json_schema() -> serde_json::Value {
        let mut schema = Self::spec().to_json_schema();
        if let Some(obj) = schema.as_object_mut() {
            obj.insert(
                "$schema".into(),
                "https://json-schema.org/draft/2020-12/schema".into(),
            );
        }
        schema
    }
}

// --- Primitive Impls ---
impl RodSchema for String {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(string())
    }
    fn spec() -> RodSpec {
        RodSpec::String {
            min: None,
            max: None,
            length: None,
            email: None,
            url: None,
            uuid: None,
            cuid: None,
            datetime: None,
            ip: None,
            regex: None,
            starts_with: None,
            ends_with: None,
            includes: None,
            trim: false,
        }
    }
}
impl RodSchema for bool {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(boolean())
    }
    fn spec() -> RodSpec {
        RodSpec::Boolean
    }
}
macro_rules! impl_int {
    ($($t:ty),*) => { $( impl RodSchema for $t {
        fn schema() -> Box<dyn RodValidator> { Box::new(number().int()) }
        fn spec() -> RodSpec { RodSpec::Number { min: None, max: None, int: Some(true) } }
    } )* };
}
impl_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl RodSchema for f32 {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(number())
    }
    fn spec() -> RodSpec {
        RodSpec::Number {
            min: None,
            max: None,
            int: Some(false),
        }
    }
}
impl RodSchema for f64 {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(number())
    }
    fn spec() -> RodSpec {
        RodSpec::Number {
            min: None,
            max: None,
            int: Some(false),
        }
    }
}

// --- Smart Pointers ---
impl<T: RodSchema> RodSchema for Box<T> {
    fn schema() -> Box<dyn RodValidator> {
        T::schema()
    }
    fn spec() -> RodSpec {
        T::spec()
    }
}
impl<T: RodSchema> RodSchema for Rc<T> {
    fn schema() -> Box<dyn RodValidator> {
        T::schema()
    }
    fn spec() -> RodSpec {
        T::spec()
    }
}
impl<T: RodSchema> RodSchema for Arc<T> {
    fn schema() -> Box<dyn RodValidator> {
        T::schema()
    }
    fn spec() -> RodSpec {
        T::spec()
    }
}
impl<'a, T: RodSchema + Clone> RodSchema for Cow<'a, T> {
    fn schema() -> Box<dyn RodValidator> {
        T::schema()
    }
    fn spec() -> RodSpec {
        T::spec()
    }
}

// --- Option & Vec ---
impl<T: RodSchema> RodSchema for Option<T> {
    fn schema() -> Box<dyn RodValidator> {
        use crate::types::nullable::NullableExtension;
        use crate::types::optional::OptionalExtension;
        Box::new(T::schema().nullable().optional())
    }
    fn spec() -> RodSpec {
        RodSpec::Optional(Box::new(RodSpec::Nullable(Box::new(T::spec()))))
    }
}
impl<T: RodSchema> RodSchema for Vec<T> {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(crate::array(T::schema()))
    }
    fn spec() -> RodSpec {
        RodSpec::Array {
            items: Box::new(T::spec()),
            min: None,
            max: None,
        }
    }
}

// --- HashMaps & HashSets ---
impl<T: RodSchema> RodSchema for HashSet<T> {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(crate::set(T::schema()))
    }
    fn spec() -> RodSpec {
        RodSpec::Set {
            value: Box::new(T::spec()),
            min: None,
        }
    }
}
impl<T: RodSchema> RodSchema for BTreeSet<T> {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(crate::set(T::schema()))
    }
    fn spec() -> RodSpec {
        RodSpec::Set {
            value: Box::new(T::spec()),
            min: None,
        }
    }
}
impl<K: RodSchema, V: RodSchema> RodSchema for HashMap<K, V> {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(crate::record(K::schema(), V::schema()))
    }
    fn spec() -> RodSpec {
        RodSpec::Record {
            key: Box::new(K::spec()),
            value: Box::new(V::spec()),
        }
    }
}
impl<K: RodSchema, V: RodSchema> RodSchema for BTreeMap<K, V> {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(crate::record(K::schema(), V::schema()))
    }
    fn spec() -> RodSpec {
        RodSpec::Record {
            key: Box::new(K::spec()),
            value: Box::new(V::spec()),
        }
    }
}

impl RodSchema for serde_json::Value {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(any())
    }
    fn spec() -> RodSpec {
        RodSpec::Any
    }
}
