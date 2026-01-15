use pyo3::prelude::*;

pub mod addresses;
pub mod async_test;
pub mod cache_demo;
pub mod constants;
pub mod drift_client;
pub mod math;
pub mod pyth_lazer;
pub mod utils;

fn init_observability() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        #[cfg(feature = "tokio-console")]
        {
            console_subscriber::Builder::default()
                .with_default_env()
                .init();
        }

        #[cfg(all(feature = "observability", not(feature = "tokio-console")))]
        {
            let filter = tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_thread_ids(true)
                .with_thread_names(true)
                .try_init();
        }
    });
}

#[pymodule]
fn _driftpyrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_observability();
    let _ = pyo3_async_runtimes::tokio::get_runtime();

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
    m.add_function(wrap_pyfunction!(utils::debug_current_thread, m)?)?;
    m.add_function(wrap_pyfunction!(utils::build_info, m)?)?;

    m.add_function(wrap_pyfunction!(async_test::sleep_and_return, m)?)?;

    m.add_class::<cache_demo::CacheDemo>()?;
    m.add_class::<drift_client::DriftClient>()?;

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
