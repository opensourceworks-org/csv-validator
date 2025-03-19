use pyo3::prelude::*;
use pyo3::types::{PyModule};

#[pyfunction]
pub fn jahallo(py: Python) -> PyResult<String>{
    Ok("jahallo".to_string())
}

// main entrypoint for python module
#[pymodule]
fn csv_validators(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(jahallo, m)?)?;
    Ok(())
}