use drift_rs::constants;
use pyo3::prelude::*;

#[pyfunction]
pub fn get_program_id() -> String {
    constants::PROGRAM_ID.to_string()
}

#[pyfunction]
pub fn get_vault_program_id() -> String {
    constants::VAULT_PROGRAM_ID.to_string()
}

#[pyfunction]
pub fn get_jit_proxy_id() -> String {
    constants::JIT_PROXY_ID.to_string()
}

#[pyfunction]
pub fn get_token_program_id() -> String {
    constants::TOKEN_PROGRAM_ID.to_string()
}

#[pyfunction]
pub fn get_token_2022_program_id() -> String {
    constants::TOKEN_2022_PROGRAM_ID.to_string()
}

#[pyfunction]
pub fn get_associated_token_program_id() -> String {
    constants::ASSOCIATED_TOKEN_PROGRAM_ID.to_string()
}

#[pyfunction]
pub fn get_state_account() -> String {
    constants::state_account().to_string()
}

#[pyfunction]
pub fn derive_spot_market_account(market_index: u16) -> String {
    constants::derive_spot_market_account(market_index).to_string()
}

#[pyfunction]
pub fn derive_perp_market_account(market_index: u16) -> String {
    constants::derive_perp_market_account(market_index).to_string()
}

#[pyfunction]
pub fn derive_spot_market_vault(market_index: u16) -> String {
    constants::derive_spot_market_vault(market_index).to_string()
}

#[pyfunction]
pub fn derive_drift_signer() -> String {
    constants::derive_drift_signer().to_string()
}
