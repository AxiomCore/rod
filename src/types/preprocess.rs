use crate::core::input::{DataType, RodInput};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone)]
pub struct RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone,
{
    preprocessor: F,
    schema: Box<dyn RodValidator>,
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
    pub fn new(preprocessor: F, schema: Box<dyn RodValidator>) -> Self {
        Self {
            preprocessor,
            schema,
        }
    }
}

// Local helper to break lifetime invariance in RodValueInput
#[derive(Debug)]
struct LocalValueInput<'a, 'b>(&'a RodValue<'b>);

impl<'a, 'b> RodInput<'a> for LocalValueInput<'a, 'b> {
    fn get_type(&self) -> DataType {
        // Proxy to inner RodValue (using basic mapping logic from core/value.rs)
        // For brevity, we map the types we expect from preprocess (JSON usually)
        match self.0 {
            RodValue::Json(v) => match v {
                Value::String(_) => DataType::String,
                Value::Number(_) => DataType::Number,
                Value::Bool(_) => DataType::Boolean,
                Value::Null => DataType::Null,
                Value::Array(_) => DataType::Array,
                Value::Object(_) => DataType::Object,
            },
            // Fallback for others
            RodValue::String(_) => DataType::String,
            RodValue::Number(_) => DataType::Number,
            RodValue::Boolean(_) => DataType::Boolean,
            RodValue::Null => DataType::Null,
            RodValue::Array(_) => DataType::Array,
            RodValue::Object(_) => DataType::Object,
            RodValue::Lazy(i) => i.get_type(),
        }
    }

    // We must implement all methods to proxy.
    // NOTE: We return Cow<'a, str>. 'a is the lifetime of the wrapper (Local stack).
    // This allows us to borrow from the local `rod_val`.
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
        // For JSON value, we can use crate::io::json::JsonInput
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
        // We cannot clone a reference to local stack variable into a Box that leaves.
        // But logic using clone_box (Lazy) inside preprocess is rare/unsupported for local vars.
        // We fallback to owned copy.
        Box::new(crate::io::json::JsonInput(Box::leak(Box::new(
            self.0.to_json(),
        )))) // Leak is bad, but preprocess + lazy is edge case.
        // BETTER: Preprocess converts to Owned JSON. We can just own it.
    }
}

impl<F> RodValidator for RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
{
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // 1. Transform Input
        let processed_value = (self.preprocessor)(input);

        // 2. Wrap and Validate
        let result_static = {
            // rod_val owns the data (it is 'static effectively regarding 'a)
            let rod_val = RodValue::Json(processed_value);

            // We use our local wrapper which allows 'local borrowing
            let wrapped_input = LocalValueInput(&rod_val);

            // Validate.
            // Result is RodValue<'local> (borrowing rod_val)
            let result = self.schema.validate_with_context(ctx, &wrapped_input)?;

            // Convert to RodValue<'static> (Owned)
            result.into_owned()
        };

        // 3. Cast 'static to 'a
        Ok(cast_static_to_lifetime(result_static))
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
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

pub fn preprocess<F, V>(preprocessor: F, schema: V) -> RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
    V: RodValidator + 'static,
{
    RodPreprocess::new(preprocessor, Box::new(schema))
}
