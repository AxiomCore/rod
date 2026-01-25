from .builders import (
    RodString,
    RodNumber,
    RodBoolean,
    RodLiteral,
    RodEnum,
    RodObject,
    RodArray,
    RodTuple,
    RodUnion,
    RodDiscriminatedUnion,
    RodType # Expose base type for type hinting
)

# --- Factory Functions ---

def string() -> RodString:
    return RodString()

def number() -> RodNumber:
    return RodNumber()

def boolean() -> RodBoolean:
    return RodBoolean()

def literal(value) -> RodLiteral:
    return RodLiteral(value)

def enum(*values) -> RodEnum:
    return RodEnum(list(values))

def object(shape) -> RodObject:
    return RodObject(shape)

def array(item_schema) -> RodArray:
    return RodArray(item_schema)

def tuple(*items) -> RodTuple:
    return RodTuple(items)

def union(*options) -> RodUnion:
    return RodUnion(list(options))

def discriminated_union(discriminator, options) -> RodDiscriminatedUnion:
    return RodDiscriminatedUnion(discriminator, options)

# --- Expose all names for easy import ---
__all__ = [
    "string", "number", "boolean", "literal", "enum", "object", "array",
    "tuple", "union", "discriminated_union",
    "RodType", "RodString", "RodNumber", "RodBoolean", "RodObject", "RodArray"
]