use crate::core::input::{DataType, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{IntoRodNode, RodNode};
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone)]
pub struct RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone,
{
    pub preprocessor: F,
    pub schema: Box<RodNode>,
}

impl<F> fmt::Debug for RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RodPreprocess")
            .field("preprocessor", &"<function>")
            .field("schema", &self.schema)
            .finish()
    }
}

impl<F> RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone,
{
    pub fn new(preprocessor: F, schema: RodNode) -> Self {
        Self {
            preprocessor,
            schema: Box::new(schema),
        }
    }
}

// Local helper to wrap preprocessed JSON Value into a RodInput
#[derive(Debug)]
struct LocalValueInput<'a, 'b>(&'a RodValue<'b>);

// REMOVED: impl RodThreadBounds for LocalValueInput
// It is automatically implemented via the blanket impl in input.rs

impl<'a, 'b> RodInput<'a> for LocalValueInput<'a, 'b> {
    fn get_type(&self) -> DataType {
        match self.0 {
            RodValue::Json(v) => match v {
                Value::String(_) => DataType::String,
                Value::Number(_) => DataType::Number,
                Value::Bool(_) => DataType::Boolean,
                Value::Null => DataType::Null,
                Value::Array(_) => DataType::Array,
                Value::Object(_) => DataType::Object,
            },
            RodValue::String(_) => DataType::String,
            RodValue::Number(_) => DataType::Number,
            RodValue::Boolean(_) => DataType::Boolean,
            RodValue::Null => DataType::Null,
            RodValue::Array(_) => DataType::Array,
            RodValue::Object(_) => DataType::Object,
            RodValue::Lazy(i) => i.get_type(),
        }
    }

    fn as_str(&self) -> Option<Cow<'a, str>> {
        self.0.as_str().map(|s| Cow::Borrowed(s))
    }
    fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }
    fn as_i64(&self) -> Option<i64> {
        match self.0 {
            RodValue::Number(n) => Some(*n as i64),
            RodValue::Json(Value::Number(n)) => n.as_i64(),
            _ => None,
        }
    }
    fn as_u64(&self) -> Option<u64> {
        match self.0 {
            RodValue::Json(Value::Number(n)) => n.as_u64(),
            RodValue::Number(n) => {
                if *n >= 0.0 {
                    Some(*n as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    fn with_key(
        &self,
        key: &str,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        match self.0 {
            RodValue::Json(Value::Object(map)) => map.get(key).map(|v| {
                let wrapper = crate::io::json::JsonInput(v);
                f(&wrapper)
            }),
            _ => None,
        }
    }

    fn with_index(
        &self,
        index: usize,
        f: &mut dyn FnMut(&dyn RodInput<'a>) -> Result<RodValue<'a>, ()>,
    ) -> Option<Result<RodValue<'a>, ()>> {
        match self.0 {
            RodValue::Json(Value::Array(arr)) => arr.get(index).map(|v| {
                let wrapper = crate::io::json::JsonInput(v);
                f(&wrapper)
            }),
            _ => None,
        }
    }

    fn count(&self) -> Option<usize> {
        match self.0 {
            RodValue::Json(Value::Array(a)) => Some(a.len()),
            RodValue::Json(Value::Object(o)) => Some(o.len()),
            _ => None,
        }
    }

    fn keys(&self) -> Option<Box<dyn Iterator<Item = Cow<'a, str>> + '_>> {
        match self.0 {
            RodValue::Json(Value::Object(o)) => {
                Some(Box::new(o.keys().map(|k| Cow::Borrowed(k.as_str()))))
            }
            _ => None,
        }
    }

    fn to_json(&self) -> Value {
        self.0.to_json()
    }

    fn clone_box(&self) -> Box<dyn RodInput<'a> + 'a> {
        Box::new(crate::io::json::JsonInput(Box::leak(Box::new(
            self.0.to_json(),
        ))))
    }
}

impl<F> RodValidator for RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        let processed_value = (self.preprocessor)(input);

        let result_static = {
            let rod_val = RodValue::Json(processed_value);
            let wrapped_input = LocalValueInput(&rod_val);

            // This call is now monomorphized for LocalValueInput
            let result = self.schema.validate_with_context(ctx, &wrapped_input)?;

            result.into_owned()
        };

        Ok(cast_static_to_lifetime(result_static))
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

fn cast_static_to_lifetime<'a>(v: RodValue<'static>) -> RodValue<'a> {
    match v {
        RodValue::String(c) => RodValue::String(Cow::Owned(c.into_owned())),
        RodValue::Number(n) => RodValue::Number(n),
        RodValue::Boolean(b) => RodValue::Boolean(b),
        RodValue::Null => RodValue::Null,
        RodValue::Array(arr) => {
            RodValue::Array(arr.into_iter().map(cast_static_to_lifetime).collect())
        }
        RodValue::Object(obj) => RodValue::Object(
            obj.into_iter()
                .map(|(k, v)| (Cow::Owned(k.into_owned()), cast_static_to_lifetime(v)))
                .collect(),
        ),
        RodValue::Json(j) => RodValue::Json(j),
        RodValue::Lazy(i) => RodValue::Json(i.to_json()),
    }
}

pub fn preprocess<F, V>(preprocessor: F, schema: V) -> RodNode
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
    V: IntoRodNode + 'static,
{
    RodNode::Custom(Box::new(RodPreprocess::new(
        preprocessor,
        schema.into_node(),
    )))
}

impl<F> DynValidator for RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> serde_json::Value + Send + Sync + Clone + 'static,
{
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &crate::core::input::BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}
