use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::RodResult;

#[derive(Debug, Clone)]
pub struct RodIntersection {
    left: Box<dyn RodValidator>,
    right: Box<dyn RodValidator>,
}

impl RodIntersection {
    pub fn new(left: Box<dyn RodValidator>, right: Box<dyn RodValidator>) -> Self {
        Self { left, right }
    }
}

impl RodValidator for RodIntersection {
    fn validate<'a>(&self, input: &dyn RodInput<'a>) -> RodResult<RodValue<'a>> {
        // Validate against both
        let v1 = self.left.validate(input)?;
        let v2 = self.right.validate(input)?;

        // Merging logic for objects
        match (v1, v2) {
            (RodValue::Object(mut o1), RodValue::Object(o2)) => {
                // Merge o2 into o1
                o1.extend(o2);
                Ok(RodValue::Object(o1))
            }
            (_, v2) => {
                // For primitives, they must match (effectively refining the type)
                Ok(v2)
            }
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn RodValidator> {
        use crate::types::optional::OptionalExtension;
        let partial_left = self.left.deep_partial_boxed();
        let partial_right = self.right.deep_partial_boxed();
        Box::new(RodIntersection::new(partial_left, partial_right).optional())
    }

    fn clone_box(&self) -> Box<dyn RodValidator> {
        Box::new(self.clone())
    }
}

pub fn intersection(
    left: impl RodValidator + 'static,
    right: impl RodValidator + 'static,
) -> RodIntersection {
    RodIntersection::new(Box::new(left), Box::new(right))
}
