use crate::core::input::{BoxedInput, RodInput};
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodUnion {
    pub options: Vec<RodNode>,
}

impl RodUnion {
    pub fn new(options: Vec<RodNode>) -> Self {
        Self { options }
    }
}

impl RodValidator for RodUnion {
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        for validator in &self.options {
            let mut temp_ctx = ctx.fork();
            if let Ok(val) = validator.validate_with_context(&mut temp_ctx, input) {
                return Ok(val);
            }
        }
        ctx.add_issue(RodIssueCode::InvalidUnion, "Invalid input".into());
        Err(())
    }
    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_options = self
            .options
            .iter()
            .map(|o| wrap_custom(o.deep_partial_boxed()))
            .collect();
        Box::new(RodUnion::new(partial_options).optional())
    }
    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

impl DynValidator for RodUnion {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        self.validate_with_context(ctx, &BoxedInput(input))
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

pub fn union<T: IntoRodNode>(options: Vec<T>) -> RodUnion {
    RodUnion::new(options.into_iter().map(|o| o.into_node()).collect())
}
