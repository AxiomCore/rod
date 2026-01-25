use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};
use crate::types::node::{IntoRodNode, RodNode};

#[derive(Debug, Clone)]
pub struct RodUnion {
    // UPDATED: Vec of Nodes
    options: Vec<RodNode>,
}

impl RodUnion {
    pub fn new(options: Vec<RodNode>) -> Self {
        Self { options }
    }
}

impl RodValidator for RodUnion {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        for validator in &self.options {
            // Use fork() (now optimized with SmallVec)
            let mut temp_ctx = ctx.fork();

            // HYBRID DISPATCH: Calling validate on RodNode
            if let Ok(val) = validator.validate_with_context(&mut temp_ctx, input) {
                return Ok(val);
            }
        }

        ctx.add_issue(RodIssueCode::InvalidUnion, "Invalid input".into());
        Err(())
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_options = self
            .options
            .iter()
            .map(|o| {
                // Wrap partial (Box<dyn>) into Node::Custom
                crate::types::node::wrap_custom(o.deep_partial_boxed())
            })
            .collect();
        Box::new(RodUnion::new(partial_options).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn union<T: IntoRodNode>(options: Vec<T>) -> RodUnion {
    RodUnion::new(options.into_iter().map(|o| o.into_node()).collect())
}
