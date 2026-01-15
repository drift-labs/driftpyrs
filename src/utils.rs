use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction]
pub fn http_to_ws(url: &str) -> PyResult<String> {
    drift_rs::utils::http_to_ws(url).map_err(PyValueError::new_err)
}

#[pyfunction]
pub fn get_ws_url(url: &str) -> PyResult<String> {
    drift_rs::utils::get_ws_url(url).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
pub fn get_http_url(url: &str) -> PyResult<String> {
    drift_rs::utils::get_http_url(url).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
pub fn debug_current_thread() -> PyResult<(Option<String>, String)> {
    let t = std::thread::current();
    Ok((t.name().map(ToString::to_string), format!("{:?}", t.id())))
}

#[pyfunction]
pub fn build_info(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let observability = cfg!(feature = "observability");
    let tokio_console = cfg!(feature = "tokio-console");
    let tokio_unstable = cfg!(tokio_unstable);
    let default_addr = "127.0.0.1:6669".to_string();
    let env_bind = std::env::var("TOKIO_CONSOLE_BIND").ok();

    let d = PyDict::new(py);
    d.set_item("observability", observability)?;
    d.set_item("tokio_console", tokio_console)?;
    d.set_item("tokio_unstable", tokio_unstable)?;
    d.set_item("tokio_console_default_addr", default_addr)?;
    d.set_item("tokio_console_bind_env", env_bind)?;
    Ok(d.into())
}
