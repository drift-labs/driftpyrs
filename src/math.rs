use drift_rs::types::PositionDirection;
use pyo3::prelude::*;

#[pyfunction]
pub fn standardize_price(price: u64, tick_size: u64, direction: &str) -> PyResult<u64> {
    let dir = match direction.to_lowercase().as_str() {
        "long" => PositionDirection::Long,
        "short" => PositionDirection::Short,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "direction must be 'long' or 'short'",
            ))
        }
    };
    Ok(drift_rs::math::standardize_price(price, tick_size, dir))
}

#[pyfunction]
pub fn standardize_price_i64(price: i64, tick_size: u64, direction: &str) -> PyResult<i64> {
    let dir = match direction.to_lowercase().as_str() {
        "long" => PositionDirection::Long,
        "short" => PositionDirection::Short,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "direction must be 'long' or 'short'",
            ))
        }
    };
    Ok(drift_rs::math::standardize_price_i64(price, tick_size, dir))
}

#[pyfunction]
pub fn standardize_base_asset_amount(base_asset_amount: u64, step_size: u64) -> u64 {
    drift_rs::math::standardize_base_asset_amount(base_asset_amount, step_size)
}

#[pyfunction]
pub fn standardize_base_asset_amount_ceil(base_asset_amount: u64, step_size: u64) -> u64 {
    drift_rs::math::standardize_base_asset_amount_ceil(base_asset_amount, step_size)
}
