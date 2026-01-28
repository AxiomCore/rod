use crate::schema::spec::RodSpec;
use serde_json::{Map, Value, json};

impl RodSpec {
    pub fn to_json_schema(&self) -> Value {
        let mut schema = Map::new();

        match self {
            RodSpec::String {
                min,
                max,
                email,
                url,
                uuid,
                cuid,
                regex,
                datetime,
                ip,
                ..
            } => {
                schema.insert("type".into(), "string".into());
                if let Some(n) = min {
                    schema.insert("minLength".into(), (*n).into());
                }
                if let Some(n) = max {
                    schema.insert("maxLength".into(), (*n).into());
                }

                // Formats
                if email.unwrap_or(false) {
                    schema.insert("format".into(), "email".into());
                }
                if url.unwrap_or(false) {
                    schema.insert("format".into(), "uri".into());
                }
                if uuid.unwrap_or(false) {
                    schema.insert("format".into(), "uuid".into());
                }
                if datetime.unwrap_or(false) {
                    schema.insert("format".into(), "date-time".into());
                }
                if ip.unwrap_or(false) {
                    schema.insert("format".into(), "idn-address".into());
                }

                // CUID mapping (as requested: pattern based)
                if cuid.unwrap_or(false) {
                    schema.insert("pattern".into(), "^[cC][^\\s-]{8,}$".into());
                }

                // Custom Regex
                if let Some(pattern) = regex {
                    schema.insert("pattern".into(), pattern.clone().into());
                }
            }

            RodSpec::Number { min, max, int } => {
                let type_name = if int.unwrap_or(false) {
                    "integer"
                } else {
                    "number"
                };
                schema.insert("type".into(), type_name.into());
                if let Some(n) = min {
                    schema.insert("minimum".into(), (*n).into());
                }
                if let Some(n) = max {
                    schema.insert("maximum".into(), (*n).into());
                }
            }

            RodSpec::Boolean => {
                schema.insert("type".into(), "boolean".into());
            }

            RodSpec::Array { items, min, max } => {
                schema.insert("type".into(), "array".into());
                schema.insert("items".into(), items.to_json_schema());
                if let Some(n) = min {
                    schema.insert("minItems".into(), (*n).into());
                }
                if let Some(n) = max {
                    schema.insert("maxItems".into(), (*n).into());
                }
            }

            RodSpec::Object { properties, strict } => {
                schema.insert("type".into(), "object".into());

                let mut props_map = Map::new();
                let mut required = Vec::new();

                for (name, spec) in properties {
                    props_map.insert(name.clone(), spec.to_json_schema());

                    // In JSON Schema, required is property of the parent object.
                    // If a spec is NOT Optional, it is required.
                    if !spec.is_optional() {
                        required.push(name.clone());
                    }
                }

                required.sort();

                schema.insert("properties".into(), Value::Object(props_map));
                if !required.is_empty() {
                    schema.insert(
                        "required".into(),
                        Value::Array(required.into_iter().map(Value::String).collect()),
                    );
                }

                if strict.unwrap_or(false) {
                    schema.insert("additionalProperties".into(), false.into());
                }
            }

            RodSpec::Tuple { items } => {
                schema.insert("type".into(), "array".into());
                let prefix_items: Vec<Value> = items.iter().map(|i| i.to_json_schema()).collect();
                schema.insert("prefixItems".into(), Value::Array(prefix_items));
                schema.insert("minItems".into(), items.len().into());
                schema.insert("maxItems".into(), items.len().into());
                schema.insert("items".into(), false.into());
            }

            RodSpec::Literal { value } => {
                schema.insert("const".into(), value.clone());
            }

            RodSpec::Enum { values } => {
                schema.insert(
                    "enum".into(),
                    Value::Array(values.iter().map(|v| Value::String(v.clone())).collect()),
                );
            }

            RodSpec::Date { .. } => {
                schema.insert("type".into(), "integer".into());
                schema.insert(
                    "description".into(),
                    "Unix timestamp in milliseconds".into(),
                );
            }

            RodSpec::Nullable(inner) => {
                let mut inner_val = inner.to_json_schema();
                if let Some(obj) = inner_val.as_object_mut() {
                    if let Some(t) = obj.get("type") {
                        let mut types = vec![t.clone()];
                        types.push("null".into());
                        obj.insert("type".into(), Value::Array(types));
                    } else {
                        return json!({ "anyOf": [inner_val, { "type": "null" }] });
                    }
                }
                return inner_val;
            }

            RodSpec::Optional(inner) => {
                return inner.to_json_schema();
            }

            RodSpec::Any => return json!({}),
            RodSpec::Never => return json!({ "not": {} }),

            // Catch-all for other recursive variants
            RodSpec::Record { value, .. } => {
                schema.insert("type".into(), "object".into());
                schema.insert("additionalProperties".into(), value.to_json_schema());
            }
            _ => {
                schema.insert("type".into(), "any".into());
            }
        }

        Value::Object(schema)
    }

    pub fn is_optional(&self) -> bool {
        match self {
            RodSpec::Optional(_) => true,
            _ => false,
        }
    }
}
