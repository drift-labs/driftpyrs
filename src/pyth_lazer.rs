use pyo3::prelude::*;

#[pyfunction]
pub fn feed_id_to_perp_market_index(feed_id: u32) -> Option<u16> {
    drift_rs::constants::pyth_lazer_feed_id_to_perp_market_index(feed_id)
}

#[pyfunction]
pub fn perp_market_index_to_feed_id(market_index: u16) -> Option<u32> {
    drift_rs::constants::perp_market_index_to_pyth_lazer_feed_id(market_index)
}
