use pyo3::prelude::*;

pub mod addresses;
pub mod constants;
pub mod math;
pub mod pyth_lazer;
pub mod utils;

#[pymodule]
fn _driftpyrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(constants::get_program_id, m)?)?;
    m.add_function(wrap_pyfunction!(constants::get_vault_program_id, m)?)?;
    m.add_function(wrap_pyfunction!(constants::get_jit_proxy_id, m)?)?;
    m.add_function(wrap_pyfunction!(constants::get_token_program_id, m)?)?;
    m.add_function(wrap_pyfunction!(constants::get_token_2022_program_id, m)?)?;
    m.add_function(wrap_pyfunction!(
        constants::get_associated_token_program_id,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(constants::get_state_account, m)?)?;
    m.add_function(wrap_pyfunction!(constants::derive_spot_market_account, m)?)?;
    m.add_function(wrap_pyfunction!(constants::derive_perp_market_account, m)?)?;
    m.add_function(wrap_pyfunction!(constants::derive_spot_market_vault, m)?)?;
    m.add_function(wrap_pyfunction!(constants::derive_drift_signer, m)?)?;

    m.add_function(wrap_pyfunction!(addresses::derive_user_account, m)?)?;
    m.add_function(wrap_pyfunction!(addresses::derive_stats_account, m)?)?;
    m.add_function(wrap_pyfunction!(addresses::derive_swift_order_account, m)?)?;
    m.add_function(wrap_pyfunction!(addresses::derive_pyth_lazer_oracle, m)?)?;
    m.add_function(wrap_pyfunction!(addresses::derive_revenue_share, m)?)?;
    m.add_function(wrap_pyfunction!(addresses::derive_revenue_share_escrow, m)?)?;

    m.add_function(wrap_pyfunction!(math::standardize_price, m)?)?;
    m.add_function(wrap_pyfunction!(math::standardize_price_i64, m)?)?;
    m.add_function(wrap_pyfunction!(math::standardize_base_asset_amount, m)?)?;
    m.add_function(wrap_pyfunction!(
        math::standardize_base_asset_amount_ceil,
        m
    )?)?;

    m.add_function(wrap_pyfunction!(utils::http_to_ws, m)?)?;
    m.add_function(wrap_pyfunction!(utils::get_ws_url, m)?)?;
    m.add_function(wrap_pyfunction!(utils::get_http_url, m)?)?;

    let pyth_lazer = PyModule::new(m.py(), "pyth_lazer")?;
    pyth_lazer.add_function(wrap_pyfunction!(
        pyth_lazer::feed_id_to_perp_market_index,
        &pyth_lazer
    )?)?;
    pyth_lazer.add_function(wrap_pyfunction!(
        pyth_lazer::perp_market_index_to_feed_id,
        &pyth_lazer
    )?)?;
    m.add_submodule(&pyth_lazer)?;

    Ok(())
}
