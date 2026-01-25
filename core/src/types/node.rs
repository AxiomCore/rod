use crate::core::input::RodInput;
use crate::core::validator::{DynValidator, RodValidator};
use crate::core::value::RodValue;
use crate::error::ValidationContext;

// Imports of all concrete validator types
use crate::types::array::RodArray;
use crate::types::boolean::RodBoolean;
use crate::types::date::RodDate;
use crate::types::discriminated_union::RodDiscriminatedUnion;
use crate::types::enum_type::RodEnum;
use crate::types::intersection::RodIntersection;
use crate::types::lazy::RodLazy;
use crate::types::literal::RodLiteral;
use crate::types::map::RodMap;
use crate::types::nullable::RodNullable;
use crate::types::number::RodNumber;
use crate::types::object::RodObject;
use crate::types::optional::RodOptional;
use crate::types::primitive::{RodAny, RodNever};
use crate::types::record::RodRecord;
use crate::types::set::RodSet;
use crate::types::string::RodString;
use crate::types::tuple::RodTuple;
use crate::types::union::RodUnion;

/// The central node enum for hybrid dispatch optimization.
///
/// In Phase 3, this enum implements the generic RodValidator trait,
/// enabling the Rust compiler to generate specialized validation code
/// for specific binding inputs (Python, WASM, JSON).
#[derive(Debug, Clone)]
pub enum RodNode {
    String(RodString),
    Number(RodNumber),
    Boolean(RodBoolean),
    Object(RodObject),
    Array(RodArray),
    Optional(RodOptional),
    Nullable(RodNullable),
    Union(RodUnion),
    DiscriminatedUnion(RodDiscriminatedUnion),
    Literal(RodLiteral),
    Enum(RodEnum),
    Tuple(RodTuple),
    Record(RodRecord),
    Map(RodMap),
    Set(RodSet),
    Date(RodDate),
    Intersection(RodIntersection),
    Any(RodAny),
    Never(RodNever),
    Lazy(RodLazy),

    /// Fallback for user-defined validators.
    /// These use the "Slow Path" (Dynamic Dispatch via validate_dyn).
    Custom(Box<dyn DynValidator>),
}

impl DynValidator for RodNode {
    fn validate_dyn<'a>(
        &self,
        ctx: &mut ValidationContext,
        input: &dyn RodInput<'a>,
    ) -> Result<RodValue<'a>, ()> {
        // Bridge: Use the Sized wrapper to call the monomorphized path
        let wrapper = crate::core::input::BoxedInput(input);
        self.validate_with_context(ctx, &wrapper)
    }

    fn is_optional_dyn(&self) -> bool {
        self.is_optional()
    }
    fn deep_partial_dyn(&self) -> Box<dyn DynValidator> {
        self.deep_partial_boxed()
    }
    fn clone_dyn(&self) -> Box<dyn DynValidator> {
        self.clone_box()
    }
}

impl RodValidator for RodNode {
    /// Specialized validation. The compiler will generate a separate
    /// machine code path for every concrete Input type 'I'.
    fn validate_with_context<'a, I: RodInput<'a>>(
        &self,
        ctx: &mut ValidationContext,
        input: &I,
    ) -> Result<RodValue<'a>, ()> {
        match self {
            RodNode::String(v) => v.validate_with_context(ctx, input),
            RodNode::Number(v) => v.validate_with_context(ctx, input),
            RodNode::Boolean(v) => v.validate_with_context(ctx, input),
            RodNode::Object(v) => v.validate_with_context(ctx, input),
            RodNode::Array(v) => v.validate_with_context(ctx, input),
            RodNode::Optional(v) => v.validate_with_context(ctx, input),
            RodNode::Nullable(v) => v.validate_with_context(ctx, input),
            RodNode::Union(v) => v.validate_with_context(ctx, input),
            RodNode::DiscriminatedUnion(v) => v.validate_with_context(ctx, input),
            RodNode::Literal(v) => v.validate_with_context(ctx, input),
            RodNode::Enum(v) => v.validate_with_context(ctx, input),
            RodNode::Tuple(v) => v.validate_with_context(ctx, input),
            RodNode::Record(v) => v.validate_with_context(ctx, input),
            RodNode::Map(v) => v.validate_with_context(ctx, input),
            RodNode::Set(v) => v.validate_with_context(ctx, input),
            RodNode::Date(v) => v.validate_with_context(ctx, input),
            RodNode::Intersection(v) => v.validate_with_context(ctx, input),
            RodNode::Any(v) => v.validate_with_context(ctx, input),
            RodNode::Never(v) => v.validate_with_context(ctx, input),
            RodNode::Lazy(v) => v.validate_with_context(ctx, input),

            // BRIDGE: Type erasure happens here for custom validators
            RodNode::Custom(v) => v.validate_dyn(ctx, input),
        }
    }

    fn is_optional(&self) -> bool {
        match self {
            RodNode::Optional(_) => true,
            RodNode::Custom(v) => v.is_optional_dyn(),
            _ => false,
        }
    }

    fn deep_partial_boxed(&self) -> Box<dyn DynValidator> {
        match self {
            RodNode::String(v) => v.deep_partial_boxed(),
            RodNode::Number(v) => v.deep_partial_boxed(),
            RodNode::Boolean(v) => v.deep_partial_boxed(),
            RodNode::Object(v) => v.deep_partial_boxed(),
            RodNode::Array(v) => v.deep_partial_boxed(),
            RodNode::Optional(v) => v.deep_partial_boxed(),
            RodNode::Nullable(v) => v.deep_partial_boxed(),
            RodNode::Union(v) => v.deep_partial_boxed(),
            RodNode::DiscriminatedUnion(v) => v.deep_partial_boxed(),
            RodNode::Literal(v) => v.deep_partial_boxed(),
            RodNode::Enum(v) => v.deep_partial_boxed(),
            RodNode::Tuple(v) => v.deep_partial_boxed(),
            RodNode::Record(v) => v.deep_partial_boxed(),
            RodNode::Map(v) => v.deep_partial_boxed(),
            RodNode::Set(v) => v.deep_partial_boxed(),
            RodNode::Date(v) => v.deep_partial_boxed(),
            RodNode::Intersection(v) => v.deep_partial_boxed(),
            RodNode::Any(v) => v.deep_partial_boxed(),
            RodNode::Never(v) => v.deep_partial_boxed(),
            RodNode::Lazy(v) => v.deep_partial_boxed(),
            RodNode::Custom(v) => v.deep_partial_dyn(),
        }
    }

    fn clone_box(&self) -> Box<dyn DynValidator> {
        Box::new(self.clone())
    }
}

/// Helper trait to convert any validator into a RodNode.
pub trait IntoRodNode {
    fn into_node(self) -> RodNode;
}

// Blanket implementation for known types to avoid double-boxing
impl IntoRodNode for RodString {
    fn into_node(self) -> RodNode {
        RodNode::String(self)
    }
}
impl IntoRodNode for RodNumber {
    fn into_node(self) -> RodNode {
        RodNode::Number(self)
    }
}
impl IntoRodNode for RodBoolean {
    fn into_node(self) -> RodNode {
        RodNode::Boolean(self)
    }
}
impl IntoRodNode for RodArray {
    fn into_node(self) -> RodNode {
        RodNode::Array(self)
    }
}
impl IntoRodNode for RodObject {
    fn into_node(self) -> RodNode {
        RodNode::Object(self)
    }
}
impl IntoRodNode for RodOptional {
    fn into_node(self) -> RodNode {
        RodNode::Optional(self)
    }
}
impl IntoRodNode for RodNullable {
    fn into_node(self) -> RodNode {
        RodNode::Nullable(self)
    }
}
impl IntoRodNode for RodUnion {
    fn into_node(self) -> RodNode {
        RodNode::Union(self)
    }
}
impl IntoRodNode for RodDiscriminatedUnion {
    fn into_node(self) -> RodNode {
        RodNode::DiscriminatedUnion(self)
    }
}
impl IntoRodNode for RodLiteral {
    fn into_node(self) -> RodNode {
        RodNode::Literal(self)
    }
}
impl IntoRodNode for RodEnum {
    fn into_node(self) -> RodNode {
        RodNode::Enum(self)
    }
}
impl IntoRodNode for RodTuple {
    fn into_node(self) -> RodNode {
        RodNode::Tuple(self)
    }
}
impl IntoRodNode for RodRecord {
    fn into_node(self) -> RodNode {
        RodNode::Record(self)
    }
}
impl IntoRodNode for RodMap {
    fn into_node(self) -> RodNode {
        RodNode::Map(self)
    }
}
impl IntoRodNode for RodSet {
    fn into_node(self) -> RodNode {
        RodNode::Set(self)
    }
}
impl IntoRodNode for RodDate {
    fn into_node(self) -> RodNode {
        RodNode::Date(self)
    }
}
impl IntoRodNode for RodIntersection {
    fn into_node(self) -> RodNode {
        RodNode::Intersection(self)
    }
}
impl IntoRodNode for RodAny {
    fn into_node(self) -> RodNode {
        RodNode::Any(self)
    }
}
impl IntoRodNode for RodNever {
    fn into_node(self) -> RodNode {
        RodNode::Never(self)
    }
}
impl IntoRodNode for RodLazy {
    fn into_node(self) -> RodNode {
        RodNode::Lazy(self)
    }
}
impl IntoRodNode for RodNode {
    fn into_node(self) -> RodNode {
        self
    }
}

/// Helper for wrapping custom trait objects into a Node.
pub fn wrap_custom(v: Box<dyn DynValidator>) -> RodNode {
    RodNode::Custom(v)
}
