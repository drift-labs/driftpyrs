use pyo3::prelude::*;
use pythonize::pythonize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
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
                let obj = pythonize(py, &market).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to serialize perp market: {}",
                        e
                    ))
                })?;
                Ok(Some(obj.unbind()))
            }
            Err(_) => Ok(None),
        }
    }

    fn get_spot_market(&self, py: Python<'_>, market_index: u16) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.try_get_spot_market_account(market_index) {
            Ok(market) => {
                let obj = pythonize(py, &market).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to serialize spot market: {}",
                        e
                    ))
                })?;
                Ok(Some(obj.unbind()))
            }
            Err(_) => Ok(None),
        }
    }

    // -------------------------------------------------------------------------
    // User Account Queries
    // -------------------------------------------------------------------------

    /// Get a user account by pubkey (async)
    fn get_user_account<'py>(
        &self,
        py: Python<'py>,
        account: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let user = inner.get_user_account(&account).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get user account: {}",
                    e
                ))
            })?;

            Python::attach(|py| {
                pythonize(py, &user)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize user: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Get user stats by authority pubkey (async)
    fn get_user_stats<'py>(
        &self,
        py: Python<'py>,
        authority: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let authority = Pubkey::from_str(authority).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let stats = inner.get_user_stats(&authority).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get user stats: {}",
                    e
                ))
            })?;

            Python::attach(|py| {
                pythonize(py, &stats)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize user stats: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Get all orders for a user account (async)
    fn all_orders<'py>(&self, py: Python<'py>, account: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let orders = inner.all_orders(&account).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get orders: {}",
                    e
                ))
            })?;

            Python::attach(|py| {
                pythonize(py, &orders)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize orders: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Get all positions (perp and spot) for a user account (async)
    fn all_positions<'py>(&self, py: Python<'py>, account: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (perp_positions, spot_positions) =
                inner.all_positions(&account).await.map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to get positions: {}",
                        e
                    ))
                })?;

            Python::attach(|py| {
                let dict = pyo3::types::PyDict::new(py);
                let perp = pythonize(py, &perp_positions).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to serialize perp positions: {}",
                        e
                    ))
                })?;
                let spot = pythonize(py, &spot_positions).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to serialize spot positions: {}",
                        e
                    ))
                })?;
                dict.set_item("perp", perp)?;
                dict.set_item("spot", spot)?;
                Ok(dict.into_any().unbind())
            })
        })
    }

    /// Get unsettled perp positions for a user account (async)
    fn unsettled_positions<'py>(
        &self,
        py: Python<'py>,
        account: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let positions = inner.unsettled_positions(&account).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get unsettled positions: {}",
                    e
                ))
            })?;

            Python::attach(|py| {
                pythonize(py, &positions)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize positions: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Get a specific perp position for a user (async)
    fn perp_position<'py>(
        &self,
        py: Python<'py>,
        account: &str,
        market_index: u16,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let position = inner
                .perp_position(&account, market_index)
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to get perp position: {}",
                        e
                    ))
                })?;

            Python::attach(|py| {
                pythonize(py, &position)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize perp position: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Get a specific spot position for a user (async)
    fn spot_position<'py>(
        &self,
        py: Python<'py>,
        account: &str,
        market_index: u16,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let account = Pubkey::from_str(account).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid pubkey: {}", e))
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let position = inner
                .spot_position(&account, market_index)
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to get spot position: {}",
                        e
                    ))
                })?;

            Python::attach(|py| {
                pythonize(py, &position)
                    .map(|obj| obj.unbind())
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to serialize spot position: {}",
                            e
                        ))
                    })
            })
        })
    }
}
