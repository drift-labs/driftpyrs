use drift_rs::constants::PROGRAM_ID;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn parse_pubkey(s: &str) -> PyResult<Pubkey> {
    Pubkey::from_str(s).map_err(|e| PyValueError::new_err(format!("Invalid pubkey: {}", e)))
}

#[pyfunction]
pub fn derive_user_account(authority: &str, sub_account_id: u16) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    Ok(drift_rs::Wallet::derive_user_account(&authority, sub_account_id).to_string())
}

#[pyfunction]
pub fn derive_stats_account(authority: &str) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    Ok(drift_rs::Wallet::derive_stats_account(&authority).to_string())
}

#[pyfunction]
pub fn derive_swift_order_account(authority: &str) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    Ok(drift_rs::Wallet::derive_swift_order_account(&authority).to_string())
}

#[pyfunction]
pub fn derive_pyth_lazer_oracle(feed_id: u32) -> String {
    drift_rs::utils::derive_pyth_lazer_oracle_public_key(feed_id).to_string()
}

#[pyfunction]
pub fn derive_revenue_share(authority: &str) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    let (account, _) =
        Pubkey::find_program_address(&[&b"REV_SHARE"[..], authority.as_ref()], &PROGRAM_ID);
    Ok(account.to_string())
}

#[pyfunction]
pub fn derive_revenue_share_escrow(authority: &str) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    let (account, _) =
        Pubkey::find_program_address(&[&b"REV_ESCROW"[..], authority.as_ref()], &PROGRAM_ID);
    Ok(account.to_string())
}
