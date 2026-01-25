use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::ValidationContext;
use crate::types::node::{IntoRodNode, RodNode, wrap_custom};

#[derive(Debug, Clone)]
pub struct RodIntersection {
    pub left: Box<RodNode>,
    pub right: Box<RodNode>,
}

impl RodIntersection {
    pub fn new(left: RodNode, right: RodNode) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl RodValidator for RodIntersection {
    fn validate_with_context<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // STATIC DISPATCH
        let v1 = self.left.validate_with_context(ctx, input);

        if v1.is_err() && ctx.should_abort() {
            return Err(());
        }

        let v2 = self.right.validate_with_context(ctx, input);

        if v1.is_err() || v2.is_err() {
            return Err(());
        }

        match (v1.unwrap(), v2.unwrap()) {
            (RodValue::Object(mut o1), RodValue::Object(o2)) => {
                o1.extend(o2);
                Ok(RodValue::Object(o1))
            }
            (_, v2) => Ok(v2),
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_left = wrap_custom(self.left.deep_partial_boxed());
        let partial_right = wrap_custom(self.right.deep_partial_boxed());
        Box::new(
            RodIntersection::new(partial_left.into_node(), partial_right.into_node()).optional(),
        )
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn intersection<L: IntoRodNode, R: IntoRodNode>(left: L, right: R) -> RodIntersection {
    RodIntersection::new(left.into_node(), right.into_node())
}
