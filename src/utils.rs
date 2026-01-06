use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
pub fn http_to_ws(url: &str) -> PyResult<String> {
    drift_rs::utils::http_to_ws(url).map_err(|e| PyValueError::new_err(e))
}

#[pyfunction]
pub fn get_ws_url(url: &str) -> PyResult<String> {
    drift_rs::utils::get_ws_url(url).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
pub fn get_http_url(url: &str) -> PyResult<String> {
    drift_rs::utils::get_http_url(url).map_err(|e| PyValueError::new_err(e.to_string()))
}
