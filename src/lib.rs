use pyo3::prelude::*;

#[pyfunction]
fn get_vault_program_id() -> PyResult<String> {
    use drift_rs::constants::VAULT_PROGRAM_ID;
    Ok(VAULT_PROGRAM_ID.to_string())
}

#[pymodule]
fn _driftpyrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_vault_program_id, m)?)?;
    Ok(())
}
