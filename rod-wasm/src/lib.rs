mod lazy;

use rod::core::validator::RodValidator;
use rod::io::json::wrap;
use rod::schema::parser::RodSpec;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize)]
struct ValidationResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<rod::error::RodError>,
}

#[wasm_bindgen]
pub struct RodSchema {
    validator: Box<dyn RodValidator>,
}

#[wasm_bindgen]
impl RodSchema {
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: JsValue) -> Result<RodSchema, JsValue> {
        let spec: RodSpec = serde_wasm_bindgen::from_value(spec_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid schema definition: {}", e)))?;

        let validator = spec.build();

        Ok(RodSchema { validator })
    }

    pub fn validate_lazy(&self, data: JsValue) -> Result<JsValue, JsValue> {
        // FIX: Use the correct JsInput struct
        let input = lazy::JsInput { value: data };

        match self.validator.validate(&input) {
            Ok(rod_val) => {
                let json_output = rod_val.to_json();
                let output = ValidationResult {
                    success: true,
                    data: Some(json_output),
                    error: None,
                };
                Ok(serde_wasm_bindgen::to_value(&output)?)
            }
            Err(e) => {
                let output = ValidationResult {
                    success: false,
                    data: None,
                    error: Some(e),
                };
                Ok(serde_wasm_bindgen::to_value(&output)?)
            }
        }
    }

    pub fn validate_eager(&self, data: JsValue) -> Result<JsValue, JsValue> {
        let json_data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize input: {}", e)))?;

        let input = wrap(&json_data);

        let result = self.validator.validate(&input);

        let output = match result {
            Ok(rod_val) => {
                let json_output = rod_val.to_json();
                ValidationResult {
                    success: true,
                    data: Some(json_output),
                    error: None,
                }
            }
            Err(e) => ValidationResult {
                success: false,
                data: None,
                error: Some(e),
            },
        };
        Ok(serde_wasm_bindgen::to_value(&output)?)
    }
}
