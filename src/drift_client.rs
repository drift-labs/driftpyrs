use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass]
pub struct DriftClient {
    inner: Arc<drift_rs::DriftClient>,
}

#[pymethods]
impl DriftClient {
    #[staticmethod]
    #[pyo3(signature = (rpc_url, context="mainnet"))]
    fn connect<'py>(
        py: Python<'py>,
        rpc_url: String,
        context: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let context = context.to_owned();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            #[cfg(feature = "observability")]
            tracing::info!(rpc_url, context, "DriftClient::connect starting");

            let drift_context = match context.as_str() {
                "mainnet" => drift_rs::Context::MainNet,
                "devnet" => drift_rs::Context::DevNet,
                _ => {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Invalid context '{}', must be 'mainnet' or 'devnet'",
                        context
                    )))
                }
            };

            let rpc_client = drift_rs::RpcClient::new(rpc_url.clone());
            let dummy_pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");
            let wallet = drift_rs::Wallet::read_only(dummy_pubkey);

            let client = drift_rs::DriftClient::new(drift_context, rpc_client, wallet)
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to connect to Drift: {}",
                        e
                    ))
                })?;

            #[cfg(feature = "observability")]
            tracing::info!("DriftClient::connect completed successfully");

            Ok(DriftClient {
                inner: Arc::new(client),
            })
        })
    }

    fn context_name(&self) -> String {
        self.inner.context.name().to_string()
    }

    fn get_perp_market_count(&self) -> usize {
        self.inner.get_all_perp_market_ids().len()
    }

    fn get_perp_market_configs(&self) -> Vec<u16> {
        self.inner
            .get_all_perp_market_ids()
            .iter()
            .map(|x| x.index())
            .collect()
    }

    fn get_spot_market_count(&self) -> usize {
        self.inner.get_all_spot_market_ids().len()
    }

    fn __repr__(&self) -> String {
        format!(
            "DriftClient(context='{}', perp_markets={}, spot_markets={})",
            self.inner.context.name(),
            self.get_perp_market_count(),
            self.get_spot_market_count()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Subscribe to all markets (starts background tasks)
    ///
    /// This spawns background tasks that continuously update the internal cache
    /// with market data. After subscribing, you can use get_perp_market() and
    /// get_spot_market() to read from the cache synchronously (no await).
    ///
    /// Returns when subscription is established (background tasks are running).
    ///
    /// Example:
    ///     ```python
    ///     await client.subscribe()
    ///     # Now background tasks are updating the cache
    ///     market = client.get_perp_market(0)  # Instant, no await
    ///     ```
    fn subscribe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            #[cfg(feature = "observability")]
            tracing::info!("DriftClient::subscribe starting");

            inner.subscribe_all_markets().await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to subscribe to markets: {}",
                    e
                ))
            })?;

            inner.subscribe_all_oracles().await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to subscribe to oracles: {}",
                    e
                ))
            })?;

            #[cfg(feature = "observability")]
            tracing::info!(
                "DriftClient::subscribe completed, background tasks running (markets + oracles)"
            );

            Ok(())
        })
    }

    fn get_perp_oracle(&self, py: Python<'_>, market_index: u16) -> PyResult<Option<Py<PyAny>>> {
        match self
            .inner
            .try_get_oracle_price_data_and_slot(drift_rs::types::MarketId::perp(market_index))
        {
            Some(oracle) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("price", oracle.data.price)?;
                dict.set_item("confidence", oracle.data.confidence)?;
                dict.set_item("delay", oracle.data.delay)?;
                dict.set_item("slot", oracle.slot)?;
                Ok(Some(dict.into_any().unbind()))
            }
            None => Ok(None),
        }
    }

    fn get_spot_oracle(&self, py: Python<'_>, market_index: u16) -> PyResult<Option<Py<PyAny>>> {
        match self
            .inner
            .try_get_oracle_price_data_and_slot(drift_rs::types::MarketId::spot(market_index))
        {
            Some(oracle) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("price", oracle.data.price)?;
                dict.set_item("confidence", oracle.data.confidence)?;
                dict.set_item("delay", oracle.data.delay)?;
                dict.set_item("slot", oracle.slot)?;
                Ok(Some(dict.into_any().unbind()))
            }
            None => Ok(None),
        }
    }

    fn get_perp_market(&self, py: Python<'_>, market_index: u16) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.try_get_perp_market_account(market_index) {
            Ok(market) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("market_index", market.market_index)?;
                dict.set_item("status", market.status as u8)?;
                dict.set_item("contract_type", market.contract_type as u8)?;
                dict.set_item("contract_tier", market.contract_tier as u8)?;

                dict.set_item(
                    "amm_base_asset_reserve",
                    market.amm.base_asset_reserve.as_u128(),
                )?;
                dict.set_item(
                    "amm_quote_asset_reserve",
                    market.amm.quote_asset_reserve.as_u128(),
                )?;
                dict.set_item("amm_sqrt_k", market.amm.sqrt_k.as_u128())?;
                dict.set_item("amm_peg_multiplier", market.amm.peg_multiplier.as_u128())?;
                dict.set_item(
                    "amm_cumulative_funding_rate_long",
                    market.amm.cumulative_funding_rate_long.as_i128(),
                )?;
                dict.set_item(
                    "amm_cumulative_funding_rate_short",
                    market.amm.cumulative_funding_rate_short.as_i128(),
                )?;
                dict.set_item("amm_last_funding_rate", market.amm.last_funding_rate)?;
                dict.set_item(
                    "amm_last_funding_rate_long",
                    market.amm.last_funding_rate_long,
                )?;
                dict.set_item(
                    "amm_last_funding_rate_short",
                    market.amm.last_funding_rate_short,
                )?;
                dict.set_item(
                    "amm_base_asset_amount_with_amm",
                    market.amm.base_asset_amount_with_amm.as_i128(),
                )?;

                dict.set_item(
                    "number_of_users_with_base",
                    market.number_of_users_with_base,
                )?;
                dict.set_item("number_of_users", market.number_of_users)?;
                dict.set_item("margin_ratio_initial", market.margin_ratio_initial)?;
                dict.set_item("margin_ratio_maintenance", market.margin_ratio_maintenance)?;

                Ok(Some(dict.into_any().unbind()))
            }
            Err(_) => Ok(None),
        }
    }

    fn get_spot_market(&self, py: Python<'_>, market_index: u16) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.try_get_spot_market_account(market_index) {
            Ok(market) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("market_index", market.market_index)?;
                dict.set_item("status", market.status as u8)?;
                dict.set_item("asset_tier", market.asset_tier as u8)?;
                dict.set_item(
                    "name",
                    String::from_utf8_lossy(&market.name)
                        .trim_end_matches('\0')
                        .to_string(),
                )?;
                dict.set_item("deposit_balance", market.deposit_balance.as_u128())?;
                dict.set_item("borrow_balance", market.borrow_balance.as_u128())?;
                dict.set_item(
                    "cumulative_deposit_interest",
                    market.cumulative_deposit_interest.as_u128(),
                )?;
                dict.set_item(
                    "cumulative_borrow_interest",
                    market.cumulative_borrow_interest.as_u128(),
                )?;
                dict.set_item("decimals", market.decimals)?;
                dict.set_item("initial_asset_weight", market.initial_asset_weight)?;
                dict.set_item("maintenance_asset_weight", market.maintenance_asset_weight)?;
                dict.set_item("initial_liability_weight", market.initial_liability_weight)?;
                dict.set_item(
                    "maintenance_liability_weight",
                    market.maintenance_liability_weight,
                )?;

                Ok(Some(dict.into_any().unbind()))
            }
            Err(_) => Ok(None),
        }
    }
}
