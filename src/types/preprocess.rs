use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::error::RodResult;
use crate::io::json;
use serde_json::Value;
use std::fmt;

// Remove #[derive(Debug)] because F might not be Debug
#[derive(Clone)]
pub struct RodPreprocess<F>
where
    F: Fn(&dyn RodInput) -> Value + Send + Sync + Clone,
{
    preprocessor: F,
    schema: Box<dyn RodValidator>,
}

// Manual Debug implementation
impl<F> fmt::Debug for RodPreprocess<F>
where
    F: Fn(&dyn RodInput) -> Value + Send + Sync + Clone,
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
    F: Fn(&dyn RodInput) -> Value + Send + Sync + Clone,
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
    F: Fn(&dyn RodInput) -> Value + Send + Sync + Clone + 'static,
{
    fn validate(&self, input: &dyn RodInput) -> RodResult<Value> {
        // 1. Transform Input
        let processed_value = (self.preprocessor)(input);

        // 2. Wrap in JsonInput adapter
        let wrapped_input = json::wrap(&processed_value);

        // 3. Validate using the wrapped input
        self.schema.validate(&wrapped_input)
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        // Preprocess lost? Zod says yes.
        self.schema.deep_partial_boxed()
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn preprocess<F, V>(preprocessor: F, schema: V) -> RodPreprocess<F>
where
    F: Fn(&dyn RodInput) -> Value + Send + Sync + Clone + 'static,
    V: RodValidator + 'static,
{
    RodPreprocess::new(preprocessor, Box::new(schema))
}
