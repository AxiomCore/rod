import json
from typing import Any, Dict, List, Tuple, Union, Literal

# --- Bridge to the Rust native extension ---

# This will be lazily populated to avoid import errors if the native module isn't built yet.
_native_schema = None
_schema_cache: Dict[str, Any] = {}

def _get_compiled_schema(spec: Dict[str, Any]):
    """
    Lazily imports the native Rust module and compiles/caches the schema.
    """
    global _native_schema
    if _native_schema is None:
        try:
            # The native module is named 'rod_py' as defined in Cargo.toml
            from rod.rod_pyo3 import RodSchema
            _native_schema = RodSchema
        except ImportError:
            raise ImportError(
                "The 'rod' native extension is not installed. "
                "Please run 'pip install -e .' or 'maturin develop' in your project root."
            )
    
    # Use a stable JSON string as the cache key
    cache_key = json.dumps(spec, sort_keys=True)
    if cache_key in _schema_cache:
        return _schema_cache[cache_key]
    
    # Compile the schema by calling the Rust constructor
    compiled = _native_schema(spec)
    _schema_cache[cache_key] = compiled
    return compiled

# --- Base Class ---

class RodType:
    """Base class for all Rod schema types."""
    
    def __init__(self, type_name: str):
        self._def: Dict[str, Any] = {"type": type_name}

    def to_spec(self) -> Dict[str, Any]:
        """Returns the raw schema definition dictionary."""
        return self._def

    def parse(self, data: Any) -> Any:
        """
        Validates data against the schema.
        Raises ValueError if validation fails.
        """
        schema = _get_compiled_schema(self.to_spec())
        try:
            return schema.validate(data)
        except ValueError as e:
            # The error from Rust is a JSON string, so we parse it
            # to raise a more structured Python error if needed.
            try:
                error_detail = json.loads(str(e))
                # You could raise a custom RodError exception here
                raise ValueError(error_detail)
            except json.JSONDecodeError:
                raise e # Raise the original error if it wasn't JSON

    def safe_parse(self, data: Any) -> Dict[str, Any]:
        """Validates data safely without throwing an exception."""
        try:
            return {"success": True, "data": self.parse(data)}
        except ValueError as e:
            error_detail = json.loads(str(e)) if isinstance(e.args[0], str) else e.args[0]
            return {"success": False, "error": error_detail}

# --- Primitive Builders ---

class RodString(RodType):
    def __init__(self):
        super().__init__("string")

    def min(self, val: int): self._def["min"] = val; return self
    def max(self, val: int): self._def["max"] = val; return self
    def length(self, val: int): self._def["length"] = val; return self
    def email(self): self._def["email"] = True; return self
    def url(self): self._def["url"] = True; return self
    def uuid(self): self._def["uuid"] = True; return self
    def datetime(self): self._def["datetime"] = True; return self
    def ip(self): self._def["ip"] = True; return self
    def regex(self, pattern: str): self._def["regex"] = pattern; return self
    def trim(self): self._def["trim"] = True; return self

class RodNumber(RodType):
    def __init__(self):
        super().__init__("number")

    def min(self, val: float): self._def["min"] = val; return self
    def max(self, val: float): self._def["max"] = val; return self
    def int(self): self._def["int"] = True; return self

class RodBoolean(RodType):
    def __init__(self):
        super().__init__("boolean")

class RodLiteral(RodType):
    def __init__(self, value: Any):
        super().__init__("literal")
        self._def["value"] = value

class RodEnum(RodType):
    def __init__(self, values: List[str]):
        super().__init__("enum")
        self._def["values"] = values

# --- Collection Builders ---

class RodObject(RodType):
    def __init__(self, shape: Dict[str, RodType]):
        super().__init__("object")
        self._def["properties"] = {k: v.to_spec() for k, v in shape.items()}
    
    def strict(self):
        self._def["strict"] = True
        return self

class RodArray(RodType):
    def __init__(self, item_schema: RodType):
        super().__init__("array")
        self._def["items"] = item_schema.to_spec()

    def min(self, val: int): self._def["min"] = val; return self
    def max(self, val: int): self._def["max"] = val; return self

class RodTuple(RodType):
    def __init__(self, items: Tuple[RodType, ...]):
        super().__init__("tuple")
        self._def["items"] = [item.to_spec() for item in items]

class RodUnion(RodType):
    def __init__(self, options: List[RodType]):
        super().__init__("union")
        self._def["options"] = [opt.to_spec() for opt in options]

class RodDiscriminatedUnion(RodType):
    def __init__(self, discriminator: str, options: List[RodObject]):
        super().__init__("discriminatedUnion")
        self._def["discriminator"] = discriminator
        self._def["options"] = [opt.to_spec() for opt in options]