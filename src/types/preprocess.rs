use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::{RodValue, RodValueInput};
use crate::error::RodResult;
use serde_json::Value;
use std::fmt;

#[derive(Clone)]
pub struct RodPreprocess<F>
where
    // F must accept RodInput with ANY lifetime 'i
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

impl<F> RodValidator for RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
{
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // 1. Transform Input
        let processed_value = (self.preprocessor)(input);

        // 2. Wrap the owned JSON value into RodValue::Json
        // This 'rod_val' lives on the stack of this function.
        let rod_val = RodValue::Json(processed_value);

        // 3. Create an adapter to allow the schema to validate this RodValue
        let wrapped_input = RodValueInput(&rod_val);

        // 4. Validate
        // 'result' here is RodValue tied to the lifetime of 'rod_val' (stack).
        // It cannot be returned directly because 'rod_val' drops at end of function.
        let result = self.schema.validate(&wrapped_input)?;

        // 5. Convert to Owned
        // This copies any borrowed data into owned strings/vecs, effectively
        // changing the lifetime to 'static, which satisfies return type RodValue<'a>.
        Ok(result.into_owned())
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn preprocess<F, V>(preprocessor: F, schema: V) -> RodPreprocess<F>
where
    F: for<'i> Fn(&dyn RodInput<'i>) -> Value + Send + Sync + Clone + 'static,
    V: RodValidator + 'static,
{
    RodPreprocess::new(preprocessor, Box::new(schema))
}
