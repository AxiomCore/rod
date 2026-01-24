use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use rod::core::validator::RodValidator;
use rod::io::json::wrap;
use rod::schema::parser::RodSpec;

#[pyclass(unsendable)]
struct RodSchema {
    validator: Box<dyn RodValidator>,
}

#[pymethods]
impl RodSchema {
    #[new]
    fn new(spec_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let spec: RodSpec = depythonize(spec_dict)
            .map_err(|e| PyValueError::new_err(format!("Invalid schema definition: {}", e)))?;

        let validator = spec.build();
        Ok(RodSchema { validator })
    }

    fn validate(&self, data: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        // 1. Convert input Python data to serde_json::Value
        let json_data: serde_json::Value = depythonize(data)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse input data: {}", e)))?;

        // 2. Create adapter
        let input = wrap(&json_data);

        // 3. VALIDATION logic - FIX: Map to owned JSON immediately to break the borrow
        // This ensures the references to `json_data` are gone before we continue
        let validation_result = self
            .validator
            .validate(&input)
            .map(|rod_val| rod_val.to_json());

        // 4. Handle result
        match validation_result {
            Ok(result_json) => {
                let py = data.py();
                pythonize(py, &result_json)
                    .map(|bound| bound.unbind())
                    .map_err(|e| PyValueError::new_err(format!("Failed to convert result: {}", e)))
            }
            Err(e) => {
                let error_json = serde_json::to_value(&e)
                    .unwrap_or(serde_json::json!({"error": "Validation failed"}));

                Err(PyValueError::new_err(error_json.to_string()))
            }
        }
    }
}

#[pymodule]
fn rod_pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RodSchema>()?;
    Ok(())
}
