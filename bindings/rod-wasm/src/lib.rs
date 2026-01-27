mod lazy;
use rod_rs::core::validator::RodValidator;
use rod_rs::core::value::RodValue;
use rod_rs::io::json::wrap;
use rod_rs::schema::parser::RodSpec;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

fn make_err_obj(code: &str, msg: &str) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"code".into(), &code.into());
    let _ = js_sys::Reflect::set(&obj, &"message".into(), &msg.into());
    obj.into()
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
            .map_err(|e| make_err_obj("InvalidSpec", &format!("{}", e)))?;
        Ok(RodSchema {
            validator: spec.build(),
        })
    }

    fn rod_to_js(&self, val: RodValue) -> JsValue {
        match val {
            // THE PERFORMANCE KEY: Return the original JS handle instantly.
            // This makes rod.any() and object passthroughs near-instant.
            RodValue::Lazy(inner) => inner.get_js_value().unwrap_or(JsValue::NULL),
            RodValue::Object(fields) => {
                let obj = js_sys::Object::new();
                for (k, v) in fields {
                    let js_v = self.rod_to_js(v);
                    // FIX: Convert Cow<str> to &str using as_ref() before into()
                    let js_k = JsValue::from_str(k.as_ref());
                    js_sys::Reflect::set(&obj, &js_k, &js_v).unwrap();
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
            // FIX: Use as_ref() for the String variant Cow too
            RodValue::String(s) => JsValue::from_str(s.as_ref()),
            RodValue::Number(n) => JsValue::from_f64(n),
            RodValue::Boolean(b) => JsValue::from_bool(b),
            RodValue::Null => JsValue::NULL,
            RodValue::Json(j) => serde_wasm_bindgen::to_value(&j).unwrap_or(JsValue::NULL),
        }
    }

    pub fn check_batch_eager(&self, collection: JsValue) -> Result<js_sys::Uint8Array, JsValue> {
        // One-time bulk serialization (The only bridge cost we pay)
        let data: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(collection)
            .map_err(|e| make_err_obj("SerializationError", &format!("{}", e)))?;

        let len = data.len();
        let mut mask = vec![0u8; len];

        // Pure Rust execution - Zero Bridge Calls!
        for i in 0..len {
            let input = rod_rs::io::json::wrap(&data[i]);
            if self.validator.validate(&input).is_ok() {
                mask[i] = 1;
            }
        }

        Ok(js_sys::Uint8Array::from(&mask[..]))
    }

    pub fn check_batch(&self, collection: JsValue) -> Result<js_sys::Uint8Array, JsValue> {
        let array = js_sys::Array::from(&collection);
        let len = array.length();
        let mut mask = vec![0u8; len as usize];

        for i in 0..len {
            let item = array.get(i);
            let input = lazy::JsInput { value: item };
            if self.validator.validate(&input).is_ok() {
                mask[i as usize] = 1;
            }
        }

        Ok(js_sys::Uint8Array::from(&mask[..]))
    }

    pub fn validate_batch(&self, collection: JsValue) -> Result<JsValue, JsValue> {
        let array = js_sys::Array::from(&collection);
        let len = array.length();
        let results = js_sys::Array::new_with_length(len);

        for i in 0..len {
            let item = array.get(i);
            let input = lazy::JsInput {
                value: item.clone(),
            };

            match self.validator.validate(&input) {
                Ok(rod_val) => {
                    if let RodValue::Lazy(inner) = &rod_val {
                        if let Some(orig) = inner.get_js_value() {
                            results.set(i, orig);
                            continue;
                        }
                    }
                    results.set(i, serde_wasm_bindgen::to_value(&rod_val.to_json())?);
                }
                Err(_) => {
                    results.set(i, JsValue::NULL);
                }
            }
        }

        Ok(results.into())
    }

    // High-performance success wrapper (No Serde)
    fn make_success(&self, data: JsValue) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"success".into(), &JsValue::TRUE).unwrap();
        js_sys::Reflect::set(&obj, &"data".into(), &data).unwrap();
        obj.into()
    }

    fn make_error(&self, err: rod_rs::error::RodError) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"success".into(), &JsValue::FALSE).unwrap();
        let err_js = serde_wasm_bindgen::to_value(&err).unwrap();
        js_sys::Reflect::set(&obj, &"error".into(), &err_js).unwrap();
        obj.into()
    }

    pub fn validate_lazy(&self, data: JsValue) -> Result<JsValue, JsValue> {
        let input = lazy::JsInput {
            value: data.clone(),
        };

        match self.validator.validate(&input) {
            Ok(rod_val) => {
                // Use the optimized recursive converter
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
