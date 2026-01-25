mod py_input;
use py_input::PyInput;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use rod::schema::parser::RodSpec;
use rod::RodNode;
use rod::RodValidator;

#[pyclass(unsendable)]
struct RodSchema {
    // UPDATED: Store the Enum node directly to enable Phase 3 monomorphization
    validator: RodNode,
}

#[pymethods]
impl RodSchema {
    #[new]
    fn new(spec_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let spec: RodSpec = depythonize(spec_dict)
            .map_err(|e| PyValueError::new_err(format!("Invalid schema: {}", e)))?;

        Ok(RodSchema {
            // UPDATED: Build specialized Enum tree
            validator: spec.build_node(),
        })
    }

    fn validate(&self, data: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        // Zero-Copy Adapter (concrete PyInput)
        let input = PyInput(data.clone());

        // SPECIALIZED CALL:
        // validator is RodNode, input is PyInput.
        // The compiler generates a machine code path where input.as_str()
        // in RodString becomes a direct call to the PyInput implementation.
        let validation_result = self.validator.validate(&input);

        match validation_result {
            Ok(rod_val) => {
                let result_json = rod_val.to_json();
                let py = data.py();
                pythonize(py, &result_json)
                    .map(|bound| bound.unbind())
                    .map_err(|e| PyValueError::new_err(format!("Conversion failed: {}", e)))
            }
            Err(e) => {
                let error_json = serde_json::to_value(&e).unwrap();
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
