mod lazy;
use rod::core::validator::RodValidator;
use rod::core::value::RodValue;
use rod::io::json::wrap;
use rod::schema::parser::RodSpec;
use rod::RodNode;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Internal helper to create structured JS error objects for spec failures.
fn make_err_obj(code: &str, msg: &str) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"code".into(), &code.into());
    let _ = js_sys::Reflect::set(&obj, &"message".into(), &msg.into());
    obj.into()
}

#[wasm_bindgen]
pub struct RodSchema {
    /// Optimized Enum tree enabling Hybrid Dispatch and Monomorphization.
    validator: RodNode,
}

#[wasm_bindgen]
impl RodSchema {
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: JsValue) -> Result<RodSchema, JsValue> {
        let spec: RodSpec = serde_wasm_bindgen::from_value(spec_json)
            .map_err(|e| make_err_obj("InvalidSpec", &format!("{}", e)))?;

        Ok(RodSchema {
            // Phase 2/3: Build the optimized static Enum tree
            validator: spec.build_node(),
        })
    }

    /// Recursive converter from Rod internal values back to JavaScript.
    /// PERFORMANCE: Directly returns JS handles for any() or passthrough fields.
    fn rod_to_js(&self, val: RodValue) -> JsValue {
        match val {
            RodValue::Lazy(inner) => inner.get_js_value().unwrap_or(JsValue::NULL),
            RodValue::Object(fields) => {
                let obj = js_sys::Object::new();
                for (k, v) in fields {
                    let js_v = self.rod_to_js(v);
                    let js_k = JsValue::from_str(k.as_ref());
                    let _ = js_sys::Reflect::set(&obj, &js_k, &js_v);
                }
                obj.into()
            }
            RodValue::Array(items) => {
                let arr = js_sys::Array::new();
                for v in items {
                    arr.push(&self.rod_to_js(v));
                }
                arr.into()
            }
            RodValue::String(s) => JsValue::from_str(s.as_ref()),
            RodValue::Number(n) => JsValue::from_f64(n),
            RodValue::Boolean(b) => JsValue::from_bool(b),
            RodValue::Null => JsValue::NULL,
            RodValue::Json(j) => serde_wasm_bindgen::to_value(&j).unwrap_or(JsValue::NULL),
        }
    }

    /// Validates a collection by serializing it to JSON first.
    /// Optimized for heavy logic on massive primitive arrays.
    pub fn check_batch_eager(&self, collection: JsValue) -> Result<js_sys::Uint8Array, JsValue> {
        let data: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(collection)
            .map_err(|e| make_err_obj("SerializationError", &format!("{}", e)))?;

        let len = data.len();
        let mut mask = vec![0u8; len];

        for i in 0..len {
            let input = rod::io::json::wrap(&data[i]);
            // Phase 3: Monomorphized call for JsonInput
            if self.validator.validate(&input).is_ok() {
                mask[i] = 1;
            }
        }

        Ok(js_sys::Uint8Array::from(&mask[..]))
    }

    /// Validates a collection using JS Reflection.
    /// Optimized for large objects where only a few fields are checked.
    pub fn check_batch(&self, collection: JsValue) -> Result<js_sys::Uint8Array, JsValue> {
        let array = js_sys::Array::from(&collection);
        let len = array.length();
        let mut mask = vec![0u8; len as usize];

        for i in 0..len {
            let item = array.get(i);
            let input = lazy::JsInput { value: item };
            // Phase 3: Monomorphized call for JsInput
            if self.validator.validate(&input).is_ok() {
                mask[i as usize] = 1;
            }
        }

        Ok(js_sys::Uint8Array::from(&mask[..]))
    }

    /// Validates a collection and returns the transformed results.
    pub fn validate_batch(&self, collection: JsValue) -> Result<JsValue, JsValue> {
        let array = js_sys::Array::from(&collection);
        let len = array.length();
        let results = js_sys::Array::new_with_length(len);

        for i in 0..len {
            let item = array.get(i);
            let input = lazy::JsInput {
                value: item.clone(),
            };

            // Phase 3: Monomorphized call
            match self.validator.validate(&input) {
                Ok(rod_val) => {
                    let data_js = self.rod_to_js(rod_val);
                    results.set(i, data_js);
                }
                Err(_) => {
                    results.set(i, JsValue::NULL);
                }
            }
        }

        Ok(results.into())
    }

    fn make_success(&self, data: JsValue) -> JsValue {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"success".into(), &JsValue::TRUE);
        let _ = js_sys::Reflect::set(&obj, &"data".into(), &data);
        obj.into()
    }

    fn make_error(&self, err: rod::error::RodError) -> JsValue {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"success".into(), &JsValue::FALSE);
        let err_js = serde_wasm_bindgen::to_value(&err).unwrap();
        let _ = js_sys::Reflect::set(&obj, &"error".into(), &err_js);
        obj.into()
    }

    pub fn validate_lazy(&self, data: JsValue) -> Result<JsValue, JsValue> {
        let input = lazy::JsInput {
            value: data.clone(),
        };

        // Phase 3: Monomorphized call for JsInput
        match self.validator.validate(&input) {
            Ok(rod_val) => {
                let data_js = self.rod_to_js(rod_val);
                Ok(self.make_success(data_js))
            }
            Err(e) => Ok(self.make_error(e)),
        }
    }

    pub fn validate_eager(&self, data: JsValue) -> Result<JsValue, JsValue> {
        let json_data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| make_err_obj("SerializationError", &format!("{}", e)))?;

        let input = wrap(&json_data);
        // Phase 3: Monomorphized call for JsonInput
        let res = match self.validator.validate(&input) {
            Ok(rod_val) => {
                let data_js = self.rod_to_js(rod_val);
                Ok(self.make_success(data_js))
            }
            Err(e) => Ok(self.make_error(e)),
        };
        res
    }
}
