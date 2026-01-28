use crate::RodSpec;
use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssueCode, ValidationContext};

#[derive(Debug, Clone)]
pub struct RodUnion {
    options: Vec<Box<dyn RodValidator>>,
}

impl RodUnion {
    pub fn new(options: Vec<Box<dyn RodValidator>>) -> Self {
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
            // Critical: Use fork() to preserve recursion depth in trial validation
            let mut temp_ctx = ctx.fork();

            // If validation succeeds in temp context, we return immediately.
            // Temp issues are discarded.
            if let Ok(val) = validator.validate_with_context(&mut temp_ctx, input) {
                return Ok(val);
            }
        }

        ctx.add_issue(RodIssueCode::InvalidUnion, "Invalid input".into());
        Err(())
    }

    fn to_spec(&self) -> RodSpec {
        RodSpec::Union {
            options: self.options.iter().map(|o| o.to_spec()).collect(),
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_options = self
            .options
            .iter()
            .map(|o| o.deep_partial_boxed())
            .collect();
        Box::new(RodUnion::new(partial_options).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn union(options: Vec<Box<dyn RodValidator>>) -> RodUnion {
    RodUnion::new(options)
}
