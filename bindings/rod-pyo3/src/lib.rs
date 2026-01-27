mod py_input;
use py_input::PyInput;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use rod_rs::core::validator::RodValidator;
use rod_rs::schema::parser::RodSpec;

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
        // 1. Create Zero-Copy Adapter
        // This wraps the Python object directly without serialization
        let input = PyInput(data.clone());

        // 2. VALIDATION logic
        // Validation now happens directly against Python memory layout
        let validation_result = self.validator.validate(&input);

        // 3. Handle result
        match validation_result {
            Ok(rod_val) => {
                // If validation passed, we convert the result back to Python.
                // Note: For pure passthrough (lazy) values, to_json() will trigger
                // depythonize() internally. We pay the serialization cost only on success
                // and only for the returned data, not for the validation process itself.
                let result_json = rod_val.to_json();

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
